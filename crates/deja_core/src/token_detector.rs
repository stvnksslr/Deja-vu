//! Token-based clone detector implementation
//!
//! This module implements a fast token-based clone detection algorithm
//! using rolling hashes and sliding windows.

use crate::clone::{Clone, CloneGroup, CloneType};
use crate::detector::{CloneDetector, DetectionConfig, DetectionMode};
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
    pub fn register_tokenizer(
        &mut self,
        language: String,
        tokenizer: Box<dyn LanguageTokenizer>,
    ) {
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
                    TokenType::Comment | TokenType::Whitespace => return None,
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

                    let candidate = CloneCandidate {
                        file_path: file.path.clone(),
                        start_line: start_token.line,
                        end_line: end_token.line,
                        start_col: start_token.column,
                        end_col: end_token.column,
                        start_offset: start_token.start,
                        end_offset: end_token.end,
                        token_count: config.min_tokens,
                    };

                    hash_map.entry(hash).or_insert_with(Vec::new).push(candidate);
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
                    let content = self.extract_content(
                        &source_file.content,
                        candidate.start_offset,
                        candidate.end_offset,
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

            // Only keep groups with multiple instances
            if group.size() >= 2 {
                groups.push(group);
            }
        }

        groups
    }

    /// Extract content from source between offsets
    fn extract_content(&self, source: &str, start: usize, end: usize) -> String {
        source
            .get(start..end)
            .unwrap_or("")
            .to_string()
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

        Ok(groups)
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
    token_count: usize,
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
            SourceFile::new(PathBuf::from("test1.mock"), code.clone(), "mock".to_string()),
            SourceFile::new(PathBuf::from("test2.mock"), code.clone(), "mock".to_string()),
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
            SourceFile::new(PathBuf::from("test1.mock"), code.clone(), "mock".to_string()),
            SourceFile::new(PathBuf::from("test2.mock"), code.clone(), "mock".to_string()),
        ];

        let result = detector.detect(&files, &config);
        assert!(result.is_ok());
        let groups = result.unwrap();

        // Should not find clones because code is too short
        assert_eq!(groups.len(), 0);
    }
}
