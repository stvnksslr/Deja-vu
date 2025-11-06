//! AST-based clone detector for Type-3 clone detection
//!
//! This module implements clone detection using Abstract Syntax Tree comparison,
//! enabling detection of Type-3 clones (code with modifications like statement
//! insertions/deletions).

use crate::clone::{Clone, CloneGroup, CloneType};
use crate::detector::{CloneDetector, DetectionConfig};
use crate::tree_edit_distance::{edit_distance_to_similarity, tree_edit_distance_bounded, EditCosts};
use crate::SourceFile;
use anyhow::Result;
use deja_ast::Ast;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Parser trait for converting source code to AST
pub trait LanguageParser: Send + Sync {
    fn parse(&self, source: &str, source_file: &str) -> Result<Ast>;
    fn language(&self) -> &str;
}

/// AST-based clone detector for balanced mode (Type-1, Type-2, Type-3)
pub struct AstBasedDetector {
    parsers: HashMap<String, Box<dyn LanguageParser>>,
}

impl AstBasedDetector {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// Register a parser for a language
    pub fn register_parser(&mut self, language: String, parser: Box<dyn LanguageParser>) {
        self.parsers.insert(language, parser);
    }

    /// Parse a source file to AST
    fn parse_file(&self, file: &SourceFile) -> Result<Ast> {
        if let Some(parser) = self.parsers.get(&file.language) {
            parser.parse(&file.content, file.path.to_str().unwrap_or("unknown"))
        } else {
            anyhow::bail!("No parser registered for language: {}", file.language)
        }
    }

    /// Extract all meaningful subtrees from an AST
    fn extract_subtrees(&self, ast: &Ast, min_nodes: usize) -> Vec<SubtreeInfo> {
        let mut subtrees = Vec::new();
        const MAX_SUBTREE_SIZE: usize = 500; // Prevent comparing huge subtrees
        const MAX_SUBTREES_PER_FILE: usize = 100; // Limit subtrees per file

        // Enforce reasonable minimum to prevent memory exhaustion
        let effective_min_nodes = min_nodes.max(5);

        // Only look at function and class nodes, not every node
        // This dramatically reduces the number of comparisons
        for node in &ast.nodes {
            // Only extract subtrees for meaningful node types
            if !matches!(
                node.kind,
                deja_ast::NodeKind::Function | deja_ast::NodeKind::Class | deja_ast::NodeKind::Method
            ) {
                continue;
            }

            if subtrees.len() >= MAX_SUBTREES_PER_FILE {
                break; // Stop if we have too many subtrees
            }

            let size = count_subtree_nodes(ast, node.id);

            // Only include subtrees above minimum size and below maximum size
            if size < effective_min_nodes {
                continue;
            }

            if size > MAX_SUBTREE_SIZE {
                continue;
            }

            // Skip trivial nodes that are likely boilerplate
            if self.is_trivial_subtree(ast, node.id) {
                continue;
            }

            subtrees.push(SubtreeInfo {
                root_id: node.id,
                size,
                hash: self.hash_subtree(ast, node.id),
            });
        }

        subtrees
    }

    /// Check if a subtree is trivial (boilerplate)
    fn is_trivial_subtree(&self, ast: &Ast, node_id: usize) -> bool {
        use deja_ast::NodeKind;

        let node = match ast.get_node(node_id) {
            Some(n) => n,
            None => return true,
        };

        // Skip single-statement trivial nodes
        match node.kind {
            NodeKind::Pass
            | NodeKind::Break
            | NodeKind::Continue
            | NodeKind::Import
            | NodeKind::ImportFrom
            | NodeKind::Comment
            | NodeKind::Docstring => true,
            _ => false,
        }
    }

    /// Create a coarse-grained structural hash of a subtree for candidate filtering
    ///
    /// This hash groups similar code together for comparison, rather than requiring
    /// exact structural matches. We only hash high-level features like node type
    /// and approximate size, so similar functions get grouped together.
    fn hash_subtree(&self, ast: &Ast, node_id: usize) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let node = match ast.get_node(node_id) {
            Some(n) => n,
            None => return 0,
        };

        let mut hasher = DefaultHasher::new();

        // Hash the node kind (Function, Class, Method)
        format!("{:?}", node.kind).hash(&mut hasher);

        // Hash approximate size (bucketed to group similar-sized code)
        // Dividing by 5 means functions with 10-14 nodes get same bucket
        let size = count_subtree_nodes(ast, node_id);
        let size_bucket = size / 5;
        size_bucket.hash(&mut hasher);

        hasher.finish()
    }

    /// Find clone candidates using hash-based filtering
    /// Returns (candidates, parsed_files) where parsed_files contains (SourceFile, Ast) pairs
    fn find_candidates(
        &self,
        files: &[SourceFile],
        config: &DetectionConfig,
    ) -> Result<(Vec<CandidatePair>, Vec<(SourceFile, Ast)>)> {
        // Parse all files to ASTs, keeping source files alongside
        let parsed_files: Vec<(SourceFile, Ast)> = files
            .par_iter()
            .filter_map(|file| {
                match self.parse_file(file) {
                    Ok(ast) => Some((file.clone(), ast)),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to parse {}: {}",
                            file.path.display(),
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        let parse_errors = files.len() - parsed_files.len();
        if parse_errors > 0 {
            eprintln!("  Warning: {} files failed to parse and were skipped", parse_errors);
        }

        eprintln!("  Extracting subtrees from {} files...", parsed_files.len());

        // Extract all subtrees
        let min_nodes = config.min_nodes;
        let all_subtrees: Vec<(usize, SubtreeInfo)> = parsed_files
            .par_iter()
            .enumerate()
            .flat_map_iter(|(file_idx, (_file, ast))| {
                self.extract_subtrees(ast, min_nodes)
                    .into_iter()
                    .map(move |st| (file_idx, st))
            })
            .collect();

        eprintln!("  Found {} subtrees, grouping by similarity...", all_subtrees.len());

        // Group subtrees by hash for candidate pairs
        let hash_map: Arc<DashMap<u64, Vec<(usize, SubtreeInfo)>>> = Arc::new(DashMap::new());

        for (file_idx, subtree) in all_subtrees {
            hash_map
                .entry(subtree.hash)
                .or_insert_with(Vec::new)
                .push((file_idx, subtree));
        }

        // Create candidate pairs from hash groups
        let mut candidates = Vec::new();

        for entry in hash_map.iter() {
            let subtrees = entry.value();

            // Only consider groups with multiple instances
            if subtrees.len() > 1 {
                // Generate all pairs
                for i in 0..subtrees.len() {
                    for j in (i + 1)..subtrees.len() {
                        candidates.push(CandidatePair {
                            file1_idx: subtrees[i].0,
                            subtree1: subtrees[i].1.clone(),
                            file2_idx: subtrees[j].0,
                            subtree2: subtrees[j].1.clone(),
                        });
                    }
                }
            }
        }

        if !candidates.is_empty() {
            eprintln!("  Comparing {} candidate pairs with tree edit distance...", candidates.len());
        }

        Ok((candidates, parsed_files))
    }

    /// Compare candidate pairs and create clone groups
    fn compare_candidates(
        &self,
        candidates: Vec<CandidatePair>,
        parsed_files: &[(SourceFile, Ast)],
        config: &DetectionConfig,
    ) -> Vec<CloneGroup> {
        let costs = EditCosts::default();

        // Compare all candidates in parallel
        let clone_pairs: Vec<ClonePair> = candidates
            .par_iter()
            .filter_map(|candidate| {
                // Look up ASTs by file index
                let ast1 = &parsed_files[candidate.file1_idx].1;
                let ast2 = &parsed_files[candidate.file2_idx].1;

                // Calculate maximum distance for early termination based on similarity threshold
                // similarity = 1 - (distance / max_size)
                // distance_threshold = (1 - similarity_threshold) * max_size
                let max_size = candidate.subtree1.size.max(candidate.subtree2.size);
                let max_distance = ((1.0 - config.similarity_threshold) * max_size as f64).ceil();

                let distance = tree_edit_distance_bounded(
                    ast1,
                    candidate.subtree1.root_id,
                    ast2,
                    candidate.subtree2.root_id,
                    &costs,
                    max_distance,
                );

                // Skip if distance exceeded threshold (early terminated)
                if distance >= max_distance {
                    return None;
                }

                let similarity = edit_distance_to_similarity(
                    distance,
                    candidate.subtree1.size,
                    candidate.subtree2.size,
                );

                // Only keep pairs above similarity threshold
                if similarity >= config.similarity_threshold {
                    Some(ClonePair {
                        file1_idx: candidate.file1_idx,
                        file2_idx: candidate.file2_idx,
                        subtree1: candidate.subtree1.clone(),
                        subtree2: candidate.subtree2.clone(),
                        similarity,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Group similar clone pairs into clone groups
        let groups = self.group_clones(clone_pairs, parsed_files);

        // Filter out overlapping/nested clones
        self.filter_overlapping_groups(groups)
    }

    /// Group clone pairs into clone groups
    fn group_clones(&self, pairs: Vec<ClonePair>, parsed_files: &[(SourceFile, Ast)]) -> Vec<CloneGroup> {
        let mut groups = Vec::new();

        // Simple grouping by similarity score ranges
        // TODO: More sophisticated clustering algorithm
        for pair in pairs {
            // Look up file and AST by index
            let (file1, ast1) = &parsed_files[pair.file1_idx];
            let (file2, ast2) = &parsed_files[pair.file2_idx];

            let node1 = ast1.get_node(pair.subtree1.root_id);
            let node2 = ast2.get_node(pair.subtree2.root_id);

            if let (Some(n1), Some(n2)) = (node1, node2) {
                let clone1 = Clone::new(
                    file1.path.clone(),
                    n1.span.line_start,
                    n1.span.line_end,
                    n1.span.column_start,
                    n1.span.column_end,
                    file1.content[n1.span.start..n1.span.end.min(file1.content.len())].to_string(),
                );

                let clone2 = Clone::new(
                    file2.path.clone(),
                    n2.span.line_start,
                    n2.span.line_end,
                    n2.span.column_start,
                    n2.span.column_end,
                    file2.content[n2.span.start..n2.span.end.min(file2.content.len())].to_string(),
                );

                // Determine clone type based on similarity
                let clone_type = if pair.similarity >= 0.95 {
                    CloneType::Type2 // Nearly identical, minor differences
                } else {
                    CloneType::Type3 // Structural similarity with modifications
                };

                let mut group = CloneGroup::new(
                    clone_type,
                    pair.similarity,
                    format!("ast_{}", pair.similarity),
                );

                group.add_instance(clone1);
                group.add_instance(clone2);

                groups.push(group);
            }
        }

        groups
    }

    /// Filter out clone groups that are entirely contained within other larger groups
    ///
    /// When detecting clones in nested structures (like classes containing methods),
    /// we may detect both the outer structure and inner elements as separate clones.
    /// This filters out the smaller nested clones to avoid redundant reporting.
    fn filter_overlapping_groups(&self, groups: Vec<CloneGroup>) -> Vec<CloneGroup> {
        let mut filtered = Vec::new();

        for (i, group) in groups.iter().enumerate() {
            let mut is_contained = false;

            // Check if this group is contained within any other group
            for (j, other_group) in groups.iter().enumerate() {
                if i == j {
                    continue;
                }

                // Check if group is entirely contained within other_group
                if self.is_group_contained_in(group, other_group) {
                    is_contained = true;
                    break;
                }
            }

            if !is_contained {
                filtered.push(group.clone());
            }
        }

        filtered
    }

    /// Check if group A is entirely contained within group B
    ///
    /// A group is contained if all its instances are subsets of corresponding
    /// instances in the other group (same files, with line ranges fully contained).
    fn is_group_contained_in(&self, group_a: &CloneGroup, group_b: &CloneGroup) -> bool {
        // Groups must have same number of instances
        if group_a.instances.len() != group_b.instances.len() {
            return false;
        }

        // Check if every instance in A has a corresponding containing instance in B
        for clone_a in &group_a.instances {
            let has_container = group_b.instances.iter().any(|clone_b| {
                // Same file
                clone_a.file == clone_b.file
                    // A's lines are within B's lines
                    && clone_a.start_line >= clone_b.start_line
                    && clone_a.end_line <= clone_b.end_line
                    // A is smaller than B (not the same clone)
                    && (clone_a.start_line > clone_b.start_line || clone_a.end_line < clone_b.end_line)
            });

            if !has_container {
                return false;
            }
        }

        true
    }
}

impl Default for AstBasedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CloneDetector for AstBasedDetector {
    fn detect(&self, files: &[SourceFile], config: &DetectionConfig) -> Result<Vec<CloneGroup>> {
        // Find candidate clone pairs and parsed files
        let (candidates, parsed_files) = self.find_candidates(files, config)?;

        // Compare candidates and create groups
        let groups = self.compare_candidates(candidates, &parsed_files, config);

        Ok(groups)
    }

    fn name(&self) -> &str {
        "AST-Based Detector"
    }

    fn description(&self) -> &str {
        "AST-based clone detection using tree edit distance. Detects Type-1, Type-2, and Type-3 clones."
    }
}

/// Information about a subtree (does not clone AST to save memory)
#[derive(Debug, Clone)]
struct SubtreeInfo {
    root_id: usize,
    size: usize,
    hash: u64,
}

/// A candidate pair of potentially similar subtrees
#[derive(Debug, Clone)]
struct CandidatePair {
    file1_idx: usize,
    subtree1: SubtreeInfo,
    file2_idx: usize,
    subtree2: SubtreeInfo,
}

/// A confirmed clone pair
#[derive(Debug, Clone)]
struct ClonePair {
    file1_idx: usize,
    file2_idx: usize,
    subtree1: SubtreeInfo,
    subtree2: SubtreeInfo,
    similarity: f64,
}

/// Count nodes in a subtree (iterative to avoid stack overflow)
fn count_subtree_nodes(ast: &Ast, node_id: usize) -> usize {
    let mut count = 0;
    let mut stack = vec![node_id];
    const MAX_NODES_TO_COUNT: usize = 1000; // Safety limit

    while let Some(current_id) = stack.pop() {
        if count >= MAX_NODES_TO_COUNT {
            return MAX_NODES_TO_COUNT; // Return limit if exceeded
        }

        if let Some(node) = ast.get_node(current_id) {
            count += 1;
            // Add all children to stack
            for &child_id in &node.children {
                stack.push(child_id);
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct MockParser;

    impl LanguageParser for MockParser {
        fn parse(&self, _source: &str, source_file: &str) -> Result<Ast> {
            Ok(Ast::new(source_file.to_string()))
        }

        fn language(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn test_detector_creation() {
        let detector = AstBasedDetector::new();
        assert_eq!(detector.name(), "AST-Based Detector");
    }

    #[test]
    fn test_register_parser() {
        let mut detector = AstBasedDetector::new();
        detector.register_parser("mock".to_string(), Box::new(MockParser));

        let file = SourceFile::new(
            PathBuf::from("test.mock"),
            "test content".to_string(),
            "mock".to_string(),
        );

        let result = detector.parse_file(&file);
        assert!(result.is_ok());
    }
}
