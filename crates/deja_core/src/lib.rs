//! Core clone detection algorithms and abstractions for Deja-vu
//!
//! This crate provides the fundamental building blocks for code clone detection,
//! including detection strategies, similarity metrics, and result structures.

pub mod ast_detector;
pub mod clone;
pub mod detector;
pub mod hash;
pub mod similarity;
pub mod token;
pub mod token_detector;
pub mod tree_edit_distance;
pub mod utils;

pub use ast_detector::{AstBasedDetector, LanguageParser};
pub use clone::{Clone, CloneGroup, CloneType};
pub use detector::{CloneDetector, DetectionConfig, DetectionMode};
pub use similarity::SimilarityMetric;
pub use token::{LanguageTokenizer, Token, TokenizationError};
pub use token_detector::TokenBasedDetector;
pub use utils::{collect_files, file_statistics, filter_by_language, FileStats};

use std::path::PathBuf;

/// A source file to be analyzed
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: String,
}

impl SourceFile {
    pub fn new(path: PathBuf, content: String, language: String) -> Self {
        Self {
            path,
            content,
            language,
        }
    }
}

/// Result of a clone detection run
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub clones: Vec<CloneGroup>,
    pub files_analyzed: usize,
    pub duration_ms: u128,
}

impl DetectionResult {
    pub fn new(clones: Vec<CloneGroup>, files_analyzed: usize, duration_ms: u128) -> Self {
        Self {
            clones,
            files_analyzed,
            duration_ms,
        }
    }

    /// Returns the total number of clone instances across all groups
    pub fn total_clones(&self) -> usize {
        self.clones.iter().map(|g| g.instances.len()).sum()
    }
}
