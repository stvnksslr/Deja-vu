//! Core clone detection algorithms and abstractions for Deja-vu
//!
//! This crate provides the fundamental building blocks for code clone detection,
//! including detection strategies, similarity metrics, and result structures.

pub mod clone;
pub mod detector;
pub mod hash;
pub mod similarity;
pub mod token;

pub use clone::{Clone, CloneGroup, CloneType};
pub use detector::{CloneDetector, DetectionConfig, DetectionMode};
pub use similarity::SimilarityMetric;
pub use token::Token;

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
