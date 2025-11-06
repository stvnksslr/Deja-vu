//! Clone detection trait and configuration

use crate::clone::CloneGroup;
use crate::SourceFile;
use anyhow::Result;

/// Detection mode determines the algorithm and accuracy tradeoff
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMode {
    /// Fast token-based detection (Type-1, Type-2)
    Fast,
    /// Balanced AST-based detection (Type-1, Type-2, Type-3)
    Balanced,
    /// Precise graph-based detection (All types including Type-4)
    Precise,
}

/// Configuration for clone detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Detection mode (Fast, Balanced, or Precise)
    pub mode: DetectionMode,
    /// Minimum number of tokens for a clone (used by token-based detector)
    pub min_tokens: usize,
    /// Minimum number of AST nodes for a clone (used by AST-based detector)
    pub min_nodes: usize,
    /// Minimum number of lines for a clone
    pub min_lines: usize,
    /// Similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Whether to ignore comments
    pub ignore_comments: bool,
    /// Whether to ignore whitespace
    pub ignore_whitespace: bool,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            mode: DetectionMode::Balanced,
            min_tokens: 20, // Lowered from 50 to detect small to medium duplicates (10-15 lines)
            min_nodes: 10,  // Minimum AST nodes for balanced mode
            min_lines: 4,   // Lowered from 5 for better sensitivity
            similarity_threshold: 0.85,
            ignore_comments: true,
            ignore_whitespace: true,
        }
    }
}

impl DetectionConfig {
    pub fn fast() -> Self {
        Self {
            mode: DetectionMode::Fast,
            min_tokens: 30, // Higher threshold for faster processing
            min_nodes: 15,  // Not used in fast mode, but set for consistency
            min_lines: 5,
            ..Default::default()
        }
    }

    pub fn balanced() -> Self {
        Self {
            mode: DetectionMode::Balanced,
            min_nodes: 10, // Moderate threshold for AST-based detection
            ..Default::default()
        }
    }

    pub fn precise() -> Self {
        Self {
            mode: DetectionMode::Precise,
            min_tokens: 15, // Lower threshold for more precise detection
            min_nodes: 8,   // Lower threshold for AST detection
            min_lines: 3,
            similarity_threshold: 0.80,
            ..Default::default()
        }
    }
}

/// Main trait for clone detection algorithms
pub trait CloneDetector: Send + Sync {
    /// Detect clones in a set of source files
    fn detect(&self, files: &[SourceFile], config: &DetectionConfig) -> Result<Vec<CloneGroup>>;

    /// Name of this detector
    fn name(&self) -> &str;

    /// Description of this detector
    fn description(&self) -> &str;
}
