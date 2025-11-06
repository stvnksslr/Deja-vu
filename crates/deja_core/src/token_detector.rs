//! Token-based clone detector implementation
//!
//! This module implements a fast token-based clone detection algorithm
//! using rolling hashes and sliding windows.

use crate::clone::{Clone, CloneGroup, CloneType};
use crate::detector::{CloneDetector, DetectionConfig};
use crate::hash::RollingHash;
use crate::token::{LanguageTokenizer, Token, TokenType};
use crate::SourceFile;
use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Token-based clone detector for fast Type-1 and Type-2 clone detection
pub struct TokenBasedDetector {
    tokenizers: HashMap<String, Box<dyn LanguageTokenizer>>,
}

impl TokenBasedDetector {
    pub fn new() -> Self {
        Self {
            tokenizers: HashMap::new(),
        }
    }

    /// Register a tokenizer for a language
    pub fn register_tokenizer(&mut self, language: String, tokenizer: Box<dyn LanguageTokenizer>) {
        self.tokenizers.insert(language, tokenizer);
    }

    /// Tokenize a source file using a language-specific tokenizer
    fn tokenize_file(&self, file: &SourceFile) -> Result<Vec<Token>> {
        if let Some(tokenizer) = self.tokenizers.get(&file.language) {
            tokenizer
                .tokenize(&file.content)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))
        } else {
            anyhow::bail!("No tokenizer registered for language: {}", file.language)
        }
    }

    /// Normalize tokens for clone detection, tracking original indices
    fn normalize_tokens(&self, tokens: &[Token], config: &DetectionConfig) -> Vec<NormalizedToken> {
        tokens
            .iter()
            .enumerate()
            .filter_map(|(index, token)| {
                // Skip comments and whitespace if configured
                if config.ignore_comments && matches!(token.token_type, TokenType::Comment) {
                    return None;
                }
                if config.ignore_whitespace && matches!(token.token_type, TokenType::Whitespace) {
                    return None;
                }

                // Normalize identifiers for Type-2 clone detection
                let normalized = match token.token_type {
                    TokenType::Identifier => "$ID".to_string(),
                    TokenType::Literal => "$LIT".to_string(),
                    TokenType::Whitespace => return None,
                    _ => token.value.clone(),
                };

                Some(NormalizedToken {
                    value: normalized,
                    original_index: index,
                })
            })
            .collect()
    }

    /// Find clone candidates using rolling hash
    fn find_candidates(
        &self,
        files: &[SourceFile],
        config: &DetectionConfig,
    ) -> Result<HashMap<u64, Vec<CloneCandidate>>> {
        // Map from hash to list of locations with that hash
        let hash_map: Arc<DashMap<u64, Vec<CloneCandidate>>> = Arc::new(DashMap::new());

        // Process files in parallel
        files.par_iter().try_for_each(|file| -> Result<()> {
            // TODO: Get tokens from language-specific tokenizer
            // For now, create a placeholder
            let tokens = self.tokenize_file(file)?;

            if tokens.is_empty() {
                return Ok(());
            }

            let normalized = self.normalize_tokens(&tokens, config);

            if normalized.len() < config.min_tokens {
                return Ok(());
            }

            // Use rolling hash to find all windows
            let mut roller = RollingHash::new(config.min_tokens);

            for (i, norm_token) in normalized.iter().enumerate() {
                if let Some(hash) = roller.push(&norm_token.value) {
                    // Calculate window boundaries in the normalized array
                    let norm_window_start = i.saturating_sub(config.min_tokens - 1);
                    let norm_window_end = i + 1;

                    // Get the original token indices from the normalized tokens
                    let start_orig_idx = normalized[norm_window_start].original_index;
                    let end_orig_idx = normalized[norm_window_end - 1].original_index;

                    // Access original tokens using the tracked indices
                    let start_token = &tokens[start_orig_idx];
                    let end_token = &tokens[end_orig_idx];

                    // Collect all token indices in this window for filtered display
                    let token_indices: Vec<usize> = normalized[norm_window_start..norm_window_end]
                        .iter()
                        .map(|nt| nt.original_index)
                        .collect();

                    let candidate = CloneCandidate {
                        file_path: file.path.clone(),
                        start_line: start_token.line,
                        end_line: end_token.line,
                        start_col: start_token.column,
                        end_col: end_token.column,
                        start_offset: start_token.start,
                        end_offset: end_token.end,
                        token_indices,
                    };

                    hash_map.entry(hash).or_default().push(candidate);
                }
            }

            Ok(())
        })?;

        // Convert DashMap to HashMap
        let result: HashMap<u64, Vec<CloneCandidate>> = hash_map
            .iter()
            .filter(|entry| entry.value().len() > 1) // Only keep hashes with multiple occurrences
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        Ok(result)
    }

    /// Extend clone candidates to find full clone regions
    fn extend_candidates(
        &self,
        candidates: HashMap<u64, Vec<CloneCandidate>>,
        files: &[SourceFile],
        config: &DetectionConfig,
    ) -> Vec<CloneGroup> {
        let mut groups = Vec::new();

        for (hash, mut candidates) in candidates {
            // Sort candidates by file and location for consistent processing
            candidates.sort_by(|a, b| {
                a.file_path
                    .cmp(&b.file_path)
                    .then(a.start_line.cmp(&b.start_line))
            });

            // Filter by minimum line count
            candidates.retain(|c| (c.end_line - c.start_line + 1) >= config.min_lines);

            if candidates.len() < 2 {
                continue;
            }

            // Create a clone group
            let mut group = CloneGroup::new(
                CloneType::Type1, // Start with Type-1, could be refined
                1.0,              // Perfect similarity for exact token matches
                format!("{:x}", hash),
            );

            for candidate in candidates {
                // Find the source file to extract content
                if let Some(source_file) = files.iter().find(|f| f.path == candidate.file_path) {
                    // Tokenize the file to get tokens for content extraction
                    if let Ok(tokens) = self.tokenize_file(source_file) {
                        // Use filtered content extraction to exclude comments/docstrings
                        let content = self.extract_content_filtered(
                            &source_file.content,
                            &tokens,
                            &candidate.token_indices,
                        );

                        let clone = Clone::new(
                            candidate.file_path.clone(),
                            candidate.start_line,
                            candidate.end_line,
                            candidate.start_col,
                            candidate.end_col,
                            content,
                        );

                        group.add_instance(clone);
                    }
                }
            }

            // Keep groups with multiple instances
            // For same-file clones, ensure they're not overlapping (artifacts of sliding window)
            if group.size() >= 2 {
                // If all clones are in the same file, check they don't overlap
                if self.has_multiple_files(&group) || !self.has_overlapping_instances(&group) {
                    // Filter out common boilerplate patterns
                    if !self.is_boilerplate_pattern(&group) {
                        groups.push(group);
                    }
                }
            }
        }

        groups
    }

    /// Check if a clone group has instances from multiple different files
    fn has_multiple_files(&self, group: &CloneGroup) -> bool {
        if group.instances.is_empty() {
            return false;
        }

        let first_file = &group.instances[0].file;
        group
            .instances
            .iter()
            .any(|clone| &clone.file != first_file)
    }

    /// Check if clone instances overlap (for same-file clones)
    /// Overlapping clones are likely sliding window artifacts
    fn has_overlapping_instances(&self, group: &CloneGroup) -> bool {
        let instances = &group.instances;

        // Sort instances by file and line for comparison
        let mut sorted_instances = instances.clone();
        sorted_instances.sort_by(|a, b| a.file.cmp(&b.file).then(a.start_line.cmp(&b.start_line)));

        // Check each pair of consecutive instances for overlap
        for i in 0..sorted_instances.len().saturating_sub(1) {
            let current = &sorted_instances[i];
            let next = &sorted_instances[i + 1];

            // Only check overlap if same file
            if current.file == next.file {
                // Check if ranges overlap
                if current.end_line >= next.start_line {
                    return true;
                }
            }
        }

        false
    }

    /// Extract content from source between offsets (legacy method)
    fn extract_content(&self, source: &str, start: usize, end: usize) -> String {
        source.get(start..end).unwrap_or("").to_string()
    }

    /// Extract content using filtered token indices (excludes comments/docstrings)
    fn extract_content_filtered(
        &self,
        source: &str,
        tokens: &[Token],
        token_indices: &[usize],
    ) -> String {
        if token_indices.is_empty() {
            return String::new();
        }

        // Reconstruct content from the specified tokens only
        let mut content = String::new();
        let mut last_line = 0;
        let mut last_end = 0;

        for &idx in token_indices {
            if idx >= tokens.len() {
                continue;
            }

            let token = &tokens[idx];

            // Add newline if this token is on a different line
            if token.line > last_line && last_line > 0 {
                content.push('\n');
                // Add proper indentation based on column
                content.push_str(&" ".repeat(token.column));
            } else if token.start > last_end && last_end > 0 && token.line == last_line {
                // Add space between tokens on the same line
                content.push(' ');
            }

            // Add the token content
            content.push_str(&source[token.start..token.end]);

            last_line = token.line;
            last_end = token.end;
        }

        content
    }

    /// Check if a clone group represents common boilerplate code
    fn is_boilerplate_pattern(&self, group: &CloneGroup) -> bool {
        // Skip if we don't have instances to check
        if group.instances.is_empty() {
            return false;
        }

        let content = &group.instances[0].content;
        let trimmed = content.trim();

        // Filter out import-only blocks
        if self.is_import_block(trimmed) {
            return true;
        }

        // Filter out simple test class setup boilerplate
        if self.is_test_boilerplate(trimmed) {
            return true;
        }

        // Filter out simple class/function declarations with minimal body
        if self.is_simple_declaration(trimmed) {
            return true;
        }

        false
    }

    /// Check if content is primarily imports
    fn is_import_block(&self, content: &str) -> bool {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return false;
        }

        // Count import-related lines
        let import_lines = lines
            .iter()
            .filter(|line| {
                line.starts_with("import ")
                    || line.starts_with("from ")
                    || line.starts_with("using ")
                    || line.starts_with("#include")
                    || line.starts_with("require(")
                    || line.starts_with("use ")
            })
            .count();

        // If 80%+ of non-empty lines are imports, it's boilerplate
        import_lines as f64 / lines.len() as f64 > 0.8
    }

    /// Check if content is test boilerplate (simple class/function setup)
    fn is_test_boilerplate(&self, content: &str) -> bool {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() > 10 {
            return false; // Too long to be simple boilerplate
        }

        // Common test boilerplate patterns
        let test_patterns = [
            "class Test",
            "class test",
            "def test_",
            "def setUp",
            "def tearDown",
            "TestCase",
            "@pytest",
            "@unittest",
        ];

        let has_test_pattern = test_patterns
            .iter()
            .any(|pattern| content.contains(pattern));

        // If it has test patterns and is short, likely boilerplate
        has_test_pattern && lines.len() <= 8
    }

    /// Check if content is a simple declaration (class/function header with minimal logic)
    fn is_simple_declaration(&self, content: &str) -> bool {
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() || lines.len() > 12 {
            return false;
        }

        // Count lines that are just structural (declarations, pass, comments, braces)
        let structural_lines = lines
            .iter()
            .filter(|line| {
                line.starts_with("class ")
                    || line.starts_with("def ")
                    || line.starts_with("function ")
                    || line.starts_with("public ")
                    || line.starts_with("private ")
                    || line.starts_with("protected ")
                    || **line == "pass"
                    || **line == "{"
                    || **line == "}"
                    || line.starts_with("//")
                    || line.starts_with("#")
            })
            .count();

        // If 70%+ of lines are structural, it's likely simple boilerplate
        structural_lines as f64 / lines.len() as f64 > 0.7
    }

    /// Merge overlapping clone groups that represent the same duplication
    ///
    /// When using a sliding window approach, we often detect many overlapping
    /// windows of the same duplication. This method consolidates them into
    /// single clone groups representing the full extent of each duplication.
    fn merge_overlapping_clones(&self, groups: Vec<CloneGroup>) -> Vec<CloneGroup> {
        if groups.is_empty() {
            return groups;
        }

        let mut merged: Vec<CloneGroup> = Vec::new();
        let mut used = vec![false; groups.len()];

        for i in 0..groups.len() {
            if used[i] {
                continue;
            }

            // Start with this group as the base
            let mut current_group = groups[i].clone();
            used[i] = true;

            // Try to merge with other groups
            let mut merged_any = true;
            while merged_any {
                merged_any = false;

                for j in 0..groups.len() {
                    if used[j] || i == j {
                        continue;
                    }

                    // Check if groups should be merged
                    if self.should_merge_groups(&current_group, &groups[j]) {
                        current_group = self.merge_two_groups(current_group, &groups[j]);
                        used[j] = true;
                        merged_any = true;
                    }
                }
            }

            merged.push(current_group);
        }

        // Filter out groups with less than 2 instances after merging
        // (groups might have instances from the same file that got merged)
        merged.into_iter().filter(|g| g.size() >= 2).collect()
    }

    /// Check if two clone groups should be merged (they represent overlapping detections)
    fn should_merge_groups(&self, group1: &CloneGroup, group2: &CloneGroup) -> bool {
        // Groups must have the same number of instances (same files involved)
        if group1.instances.len() != group2.instances.len() {
            return false;
        }

        // Count how many instances overlap
        let mut overlap_count = 0;

        for clone1 in &group1.instances {
            for clone2 in &group2.instances {
                if clone1.file == clone2.file && self.clones_overlap(clone1, clone2) {
                    overlap_count += 1;
                    break;
                }
            }
        }

        // If most instances overlap, these groups should be merged
        // We use >= 50% threshold to handle cases where groups partially overlap
        overlap_count as f64 / group1.instances.len() as f64 >= 0.5
    }

    /// Check if two clones overlap in their line ranges
    fn clones_overlap(&self, clone1: &Clone, clone2: &Clone) -> bool {
        // Clones overlap if the maximum start is less than or equal to minimum end
        let overlap_start = clone1.start_line.max(clone2.start_line);
        let overlap_end = clone1.end_line.min(clone2.end_line);

        overlap_start <= overlap_end
    }

    /// Merge two clone groups by taking the union of their instances
    /// and extending boundaries to cover the full extent
    fn merge_two_groups(&self, mut group1: CloneGroup, group2: &CloneGroup) -> CloneGroup {
        // For each file, find the corresponding clones and merge them
        for clone2 in &group2.instances {
            // Find matching clone in group1 (same file)
            if let Some(clone1) = group1.instances.iter_mut().find(|c| c.file == clone2.file) {
                // Extend boundaries to cover both clones
                clone1.start_line = clone1.start_line.min(clone2.start_line);
                clone1.end_line = clone1.end_line.max(clone2.end_line);
                clone1.start_col = clone1.start_col.min(clone2.start_col);
                clone1.end_col = clone1.end_col.max(clone2.end_col);

                // Use the content from the larger clone (more complete)
                if clone2.content.len() > clone1.content.len() {
                    clone1.content = clone2.content.clone();
                }
            } else {
                // This clone is in a file not yet in group1, add it
                group1.instances.push(clone2.clone());
            }
        }

        group1
    }
}

impl Default for TokenBasedDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CloneDetector for TokenBasedDetector {
    fn detect(&self, files: &[SourceFile], config: &DetectionConfig) -> Result<Vec<CloneGroup>> {
        // Step 1: Find clone candidates using rolling hash
        let candidates = self.find_candidates(files, config)?;

        // Step 2: Extend candidates to find full clone regions
        let groups = self.extend_candidates(candidates, files, config);

        // Step 3: Merge overlapping clone groups
        let merged_groups = self.merge_overlapping_clones(groups);

        Ok(merged_groups)
    }

    fn name(&self) -> &str {
        "Token-Based Detector"
    }

    fn description(&self) -> &str {
        "Fast token-based clone detection using rolling hashes. Detects Type-1 and Type-2 clones."
    }
}

/// A candidate clone location
#[derive(Debug, Clone)]
struct CloneCandidate {
    file_path: std::path::PathBuf,
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
    start_offset: usize,
    end_offset: usize,
    /// Original token indices that make up this clone (for filtered display)
    token_indices: Vec<usize>,
}

/// Normalized token with original index tracking
#[derive(Debug, Clone)]
struct NormalizedToken {
    /// The normalized token string
    value: String,
    /// The index in the original tokens array
    original_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::DetectionMode;
    use crate::token::{LanguageTokenizer, TokenizationError};
    use std::path::PathBuf;

    // Mock tokenizer for testing
    struct MockTokenizer;

    impl LanguageTokenizer for MockTokenizer {
        fn tokenize(&self, source: &str) -> Result<Vec<Token>, TokenizationError> {
            // Simple mock tokenizer that splits on whitespace
            let mut tokens = Vec::new();
            let mut offset = 0;
            let mut line = 1;
            let mut col = 0;

            for word in source.split_whitespace() {
                let token_type = match word {
                    "def" | "class" | "if" | "else" | "return" => TokenType::Keyword,
                    _ if word.chars().all(|c| c.is_numeric()) => TokenType::Literal,
                    _ => TokenType::Identifier,
                };

                tokens.push(Token::new(
                    token_type,
                    word.to_string(),
                    offset,
                    offset + word.len(),
                    line,
                    col,
                ));

                offset += word.len() + 1; // +1 for space
                col += word.len() + 1;
            }

            Ok(tokens)
        }

        fn language(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn test_detector_creation() {
        let detector = TokenBasedDetector::new();
        assert_eq!(detector.name(), "Token-Based Detector");
    }

    #[test]
    fn test_normalize_tokens() {
        let detector = TokenBasedDetector::new();
        let config = DetectionConfig::default();

        let tokens = vec![
            Token::new(TokenType::Keyword, "def".to_string(), 0, 3, 1, 0),
            Token::new(TokenType::Identifier, "foo".to_string(), 4, 7, 1, 4),
            Token::new(TokenType::Literal, "42".to_string(), 10, 12, 1, 10),
        ];

        let normalized = detector.normalize_tokens(&tokens, &config);
        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0].value, "def");
        assert_eq!(normalized[0].original_index, 0);
        assert_eq!(normalized[1].value, "$ID");
        assert_eq!(normalized[1].original_index, 1);
        assert_eq!(normalized[2].value, "$LIT");
        assert_eq!(normalized[2].original_index, 2);
    }

    #[test]
    fn test_normalize_tokens_with_comments() {
        let detector = TokenBasedDetector::new();
        let mut config = DetectionConfig::default();
        config.ignore_comments = true;

        let tokens = vec![
            Token::new(TokenType::Keyword, "def".to_string(), 0, 3, 1, 0),
            Token::new(TokenType::Comment, "# comment".to_string(), 4, 13, 1, 4),
            Token::new(TokenType::Identifier, "foo".to_string(), 14, 17, 1, 14),
        ];

        let normalized = detector.normalize_tokens(&tokens, &config);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].value, "def");
        assert_eq!(normalized[0].original_index, 0); // First token
        assert_eq!(normalized[1].value, "$ID");
        assert_eq!(normalized[1].original_index, 2); // Third token (comment was skipped)
    }

    #[test]
    fn test_normalize_tokens_with_whitespace() {
        let detector = TokenBasedDetector::new();
        let mut config = DetectionConfig::default();
        config.ignore_whitespace = true;

        let tokens = vec![
            Token::new(TokenType::Keyword, "def".to_string(), 0, 3, 1, 0),
            Token::new(TokenType::Whitespace, "\n".to_string(), 3, 4, 1, 3),
            Token::new(TokenType::Identifier, "foo".to_string(), 4, 7, 2, 0),
        ];

        let normalized = detector.normalize_tokens(&tokens, &config);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].value, "def");
        assert_eq!(normalized[0].original_index, 0); // First token
        assert_eq!(normalized[1].value, "$ID");
        assert_eq!(normalized[1].original_index, 2); // Third token (whitespace was skipped)
    }

    #[test]
    fn test_extract_content() {
        let detector = TokenBasedDetector::new();
        let source = "def foo():\n    return 42";
        let content = detector.extract_content(source, 0, 10);
        assert_eq!(content, "def foo():");
    }

    #[test]
    fn test_extract_content_out_of_bounds() {
        let detector = TokenBasedDetector::new();
        let source = "short";
        let content = detector.extract_content(source, 0, 100);
        assert_eq!(content, "");
    }

    #[test]
    fn test_register_tokenizer() {
        let mut detector = TokenBasedDetector::new();
        detector.register_tokenizer("test".to_string(), Box::new(MockTokenizer));

        // Verify tokenizer was registered by trying to use it
        let source = SourceFile::new(
            PathBuf::from("test.mock"),
            "def foo bar".to_string(),
            "test".to_string(),
        );

        let result = detector.tokenize_file(&source);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_tokenize_file_no_tokenizer() {
        let detector = TokenBasedDetector::new();
        let source = SourceFile::new(
            PathBuf::from("test.unknown"),
            "content".to_string(),
            "unknown".to_string(),
        );

        let result = detector.tokenize_file(&source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No tokenizer"));
    }

    #[test]
    fn test_detect_empty_files() {
        let detector = TokenBasedDetector::new();
        let config = DetectionConfig::default();
        let files: Vec<SourceFile> = vec![];

        let result = detector.detect(&files, &config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_detect_single_file() {
        let mut detector = TokenBasedDetector::new();
        detector.register_tokenizer("mock".to_string(), Box::new(MockTokenizer));

        let config = DetectionConfig {
            mode: DetectionMode::Fast,
            min_tokens: 3,
            min_lines: 1,
            similarity_threshold: 0.8,
            ignore_comments: true,
            ignore_whitespace: true,
        };

        let files = vec![SourceFile::new(
            PathBuf::from("test1.mock"),
            "def foo bar baz qux".to_string(),
            "mock".to_string(),
        )];

        let result = detector.detect(&files, &config);
        assert!(result.is_ok());
        // Single file should not produce clones
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_detect_identical_code() {
        let mut detector = TokenBasedDetector::new();
        detector.register_tokenizer("mock".to_string(), Box::new(MockTokenizer));

        let config = DetectionConfig {
            mode: DetectionMode::Fast,
            min_tokens: 3,
            min_lines: 1,
            similarity_threshold: 0.8,
            ignore_comments: true,
            ignore_whitespace: true,
        };

        let code = "def foo bar baz qux".to_string();
        let files = vec![
            SourceFile::new(
                PathBuf::from("test1.mock"),
                code.clone(),
                "mock".to_string(),
            ),
            SourceFile::new(
                PathBuf::from("test2.mock"),
                code.clone(),
                "mock".to_string(),
            ),
        ];

        let result = detector.detect(&files, &config);
        assert!(result.is_ok());
        let groups = result.unwrap();

        // Should find at least one clone group with the identical code
        assert!(groups.len() > 0, "Should detect clones in identical code");
    }

    #[test]
    fn test_detect_respects_min_tokens() {
        let mut detector = TokenBasedDetector::new();
        detector.register_tokenizer("mock".to_string(), Box::new(MockTokenizer));

        let config = DetectionConfig {
            mode: DetectionMode::Fast,
            min_tokens: 100, // Very high threshold
            min_lines: 1,
            similarity_threshold: 0.8,
            ignore_comments: true,
            ignore_whitespace: true,
        };

        let code = "def foo bar".to_string();
        let files = vec![
            SourceFile::new(
                PathBuf::from("test1.mock"),
                code.clone(),
                "mock".to_string(),
            ),
            SourceFile::new(
                PathBuf::from("test2.mock"),
                code.clone(),
                "mock".to_string(),
            ),
        ];

        let result = detector.detect(&files, &config);
        assert!(result.is_ok());
        let groups = result.unwrap();

        // Should not find clones because code is too short
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_clones_overlap() {
        let detector = TokenBasedDetector::new();

        // Test overlapping clones
        let clone1 = Clone::new(PathBuf::from("test.rs"), 1, 10, 0, 0, "content".to_string());
        let clone2 = Clone::new(PathBuf::from("test.rs"), 5, 15, 0, 0, "content".to_string());
        assert!(detector.clones_overlap(&clone1, &clone2));

        // Test non-overlapping clones
        let clone3 = Clone::new(
            PathBuf::from("test.rs"),
            20,
            30,
            0,
            0,
            "content".to_string(),
        );
        assert!(!detector.clones_overlap(&clone1, &clone3));

        // Test adjacent clones (should overlap at boundary)
        let clone4 = Clone::new(
            PathBuf::from("test.rs"),
            10,
            20,
            0,
            0,
            "content".to_string(),
        );
        assert!(detector.clones_overlap(&clone1, &clone4));
    }

    #[test]
    fn test_should_merge_groups() {
        let detector = TokenBasedDetector::new();

        // Create two groups with overlapping instances in the same files
        let mut group1 = CloneGroup::new(CloneType::Type1, 1.0, "hash1".to_string());
        group1.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            1,
            10,
            0,
            0,
            "content".to_string(),
        ));
        group1.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            1,
            10,
            0,
            0,
            "content".to_string(),
        ));

        let mut group2 = CloneGroup::new(CloneType::Type1, 1.0, "hash2".to_string());
        group2.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            5,
            15,
            0,
            0,
            "content".to_string(),
        ));
        group2.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            5,
            15,
            0,
            0,
            "content".to_string(),
        ));

        // Should merge because they overlap in both files
        assert!(detector.should_merge_groups(&group1, &group2));

        // Create a group with different files - should not merge
        let mut group3 = CloneGroup::new(CloneType::Type1, 1.0, "hash3".to_string());
        group3.add_instance(Clone::new(
            PathBuf::from("file3.rs"),
            1,
            10,
            0,
            0,
            "content".to_string(),
        ));

        assert!(!detector.should_merge_groups(&group1, &group3));
    }

    #[test]
    fn test_merge_overlapping_clones() {
        let detector = TokenBasedDetector::new();

        // Create overlapping groups with instances in multiple files
        let mut group1 = CloneGroup::new(CloneType::Type1, 1.0, "hash1".to_string());
        group1.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            1,
            10,
            0,
            50,
            "content1".to_string(),
        ));
        group1.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            1,
            10,
            0,
            50,
            "content1".to_string(),
        ));

        let mut group2 = CloneGroup::new(CloneType::Type1, 1.0, "hash2".to_string());
        group2.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            5,
            15,
            0,
            50,
            "longer content2".to_string(),
        ));
        group2.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            5,
            15,
            0,
            50,
            "longer content2".to_string(),
        ));

        let groups = vec![group1, group2];
        let merged = detector.merge_overlapping_clones(groups);

        // Should merge into one group
        assert_eq!(merged.len(), 1);

        // The merged group should have extended boundaries in both files
        let file1_clone = merged[0]
            .instances
            .iter()
            .find(|c| c.file == PathBuf::from("file1.rs"))
            .unwrap();
        assert_eq!(file1_clone.start_line, 1); // min of 1 and 5
        assert_eq!(file1_clone.end_line, 15); // max of 10 and 15
        assert_eq!(file1_clone.content, "longer content2");

        let file2_clone = merged[0]
            .instances
            .iter()
            .find(|c| c.file == PathBuf::from("file2.rs"))
            .unwrap();
        assert_eq!(file2_clone.start_line, 1);
        assert_eq!(file2_clone.end_line, 15);
    }

    #[test]
    fn test_merge_non_overlapping_clones() {
        let detector = TokenBasedDetector::new();

        // Create non-overlapping groups with instances in multiple files
        let mut group1 = CloneGroup::new(CloneType::Type1, 1.0, "hash1".to_string());
        group1.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            1,
            10,
            0,
            0,
            "content1".to_string(),
        ));
        group1.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            1,
            10,
            0,
            0,
            "content1".to_string(),
        ));

        let mut group2 = CloneGroup::new(CloneType::Type1, 1.0, "hash2".to_string());
        group2.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            20,
            30,
            0,
            0,
            "content2".to_string(),
        ));
        group2.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            20,
            30,
            0,
            0,
            "content2".to_string(),
        ));

        let groups = vec![group1, group2];
        let merged = detector.merge_overlapping_clones(groups);

        // Should keep as separate groups since they don't overlap
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_multiple_overlapping_groups() {
        let detector = TokenBasedDetector::new();

        // Create three overlapping groups with instances in multiple files
        let mut group1 = CloneGroup::new(CloneType::Type1, 1.0, "hash1".to_string());
        group1.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            1,
            10,
            0,
            0,
            "content1".to_string(),
        ));
        group1.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            1,
            10,
            0,
            0,
            "content1".to_string(),
        ));

        let mut group2 = CloneGroup::new(CloneType::Type1, 1.0, "hash2".to_string());
        group2.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            5,
            15,
            0,
            0,
            "content2".to_string(),
        ));
        group2.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            5,
            15,
            0,
            0,
            "content2".to_string(),
        ));

        let mut group3 = CloneGroup::new(CloneType::Type1, 1.0, "hash3".to_string());
        group3.add_instance(Clone::new(
            PathBuf::from("file1.rs"),
            10,
            20,
            0,
            0,
            "content3".to_string(),
        ));
        group3.add_instance(Clone::new(
            PathBuf::from("file2.rs"),
            10,
            20,
            0,
            0,
            "content3".to_string(),
        ));

        let groups = vec![group1, group2, group3];
        let merged = detector.merge_overlapping_clones(groups);

        // All three should merge into one
        assert_eq!(merged.len(), 1);

        // The merged group should span from 1 to 20 in both files
        let file1_clone = merged[0]
            .instances
            .iter()
            .find(|c| c.file == PathBuf::from("file1.rs"))
            .unwrap();
        assert_eq!(file1_clone.start_line, 1);
        assert_eq!(file1_clone.end_line, 20);

        let file2_clone = merged[0]
            .instances
            .iter()
            .find(|c| c.file == PathBuf::from("file2.rs"))
            .unwrap();
        assert_eq!(file2_clone.start_line, 1);
        assert_eq!(file2_clone.end_line, 20);
    }
}
