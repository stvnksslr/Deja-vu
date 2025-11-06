//! AST-based clone detector for Type-3 clone detection
//!
//! This module implements clone detection using Abstract Syntax Tree comparison,
//! enabling detection of Type-3 clones (code with modifications like statement
//! insertions/deletions).

use crate::clone::{Clone, CloneGroup, CloneType};
use crate::detector::{CloneDetector, DetectionConfig};
use crate::tree_edit_distance::{edit_distance_to_similarity, tree_edit_distance, EditCosts};
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

        // Walk the AST and extract subtrees that are large enough
        for node in &ast.nodes {
            let size = count_subtree_nodes(ast, node.id);

            // Only include subtrees above minimum size
            if size >= min_nodes {
                // Skip trivial nodes that are likely boilerplate
                if !self.is_trivial_subtree(ast, node.id) {
                    subtrees.push(SubtreeInfo {
                        ast: ast.clone(),
                        root_id: node.id,
                        size,
                        hash: self.hash_subtree(ast, node.id),
                    });
                }
            }
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

    /// Create a structural hash of a subtree for candidate filtering
    fn hash_subtree(&self, ast: &Ast, node_id: usize) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let node = match ast.get_node(node_id) {
            Some(n) => n,
            None => return 0,
        };

        let mut hasher = DefaultHasher::new();

        // Hash the node kind
        format!("{:?}", node.kind).hash(&mut hasher);

        // Hash children recursively (limited depth for performance)
        for (i, &child_id) in node.children.iter().enumerate() {
            if i < 5 {
                // Limit to first 5 children for hashing
                self.hash_subtree(ast, child_id).hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    /// Find clone candidates using hash-based filtering
    fn find_candidates(
        &self,
        files: &[SourceFile],
        config: &DetectionConfig,
    ) -> Result<Vec<CandidatePair>> {
        // Parse all files to ASTs
        let asts: Vec<(SourceFile, Ast)> = files
            .par_iter()
            .filter_map(|file| {
                match self.parse_file(file) {
                    Ok(ast) => Some((file.clone(), ast)),
                    Err(_) => None, // Skip files that fail to parse
                }
            })
            .collect();

        // Extract all subtrees
        let min_nodes = config.min_tokens / 5; // Rough heuristic: tokens ~= 5 * nodes
        let all_subtrees: Vec<(usize, SubtreeInfo)> = asts
            .par_iter()
            .enumerate()
            .flat_map(|(file_idx, (_file, ast))| {
                self.extract_subtrees(ast, min_nodes)
                    .into_iter()
                    .map(move |st| (file_idx, st))
            })
            .collect();

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

        Ok(candidates)
    }

    /// Compare candidate pairs and create clone groups
    fn compare_candidates(
        &self,
        candidates: Vec<CandidatePair>,
        files: &[SourceFile],
        config: &DetectionConfig,
    ) -> Vec<CloneGroup> {
        let costs = EditCosts::default();

        // Compare all candidates in parallel
        let clone_pairs: Vec<ClonePair> = candidates
            .par_iter()
            .filter_map(|candidate| {
                let ast1 = &candidate.subtree1.ast;
                let ast2 = &candidate.subtree2.ast;

                let distance = tree_edit_distance(
                    ast1,
                    candidate.subtree1.root_id,
                    ast2,
                    candidate.subtree2.root_id,
                    &costs,
                );

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
        self.group_clones(clone_pairs, files)
    }

    /// Group clone pairs into clone groups
    fn group_clones(&self, pairs: Vec<ClonePair>, files: &[SourceFile]) -> Vec<CloneGroup> {
        let mut groups = Vec::new();

        // Simple grouping by similarity score ranges
        // TODO: More sophisticated clustering algorithm
        for pair in pairs {
            let node1 = pair.subtree1.ast.get_node(pair.subtree1.root_id);
            let node2 = pair.subtree2.ast.get_node(pair.subtree2.root_id);

            if let (Some(n1), Some(n2)) = (node1, node2) {
                let file1 = &files[pair.file1_idx];
                let file2 = &files[pair.file2_idx];

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
}

impl Default for AstBasedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CloneDetector for AstBasedDetector {
    fn detect(&self, files: &[SourceFile], config: &DetectionConfig) -> Result<Vec<CloneGroup>> {
        // Find candidate clone pairs
        let candidates = self.find_candidates(files, config)?;

        // Compare candidates and create groups
        let groups = self.compare_candidates(candidates, files, config);

        Ok(groups)
    }

    fn name(&self) -> &str {
        "AST-Based Detector"
    }

    fn description(&self) -> &str {
        "AST-based clone detection using tree edit distance. Detects Type-1, Type-2, and Type-3 clones."
    }
}

/// Information about a subtree
#[derive(Debug, Clone)]
struct SubtreeInfo {
    ast: Ast,
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

/// Count nodes in a subtree
fn count_subtree_nodes(ast: &Ast, node_id: usize) -> usize {
    let node = match ast.get_node(node_id) {
        Some(n) => n,
        None => return 0,
    };

    let mut count = 1;
    for &child_id in &node.children {
        count += count_subtree_nodes(ast, child_id);
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
