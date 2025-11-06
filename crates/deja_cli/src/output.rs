//! Output formatting for detection results

use deja_core::{CloneGroup, DetectionConfig};
use serde::{Deserialize, Serialize};

/// Structured JSON output format
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Tool metadata
    pub metadata: ToolMetadata,
    /// Detection configuration used
    pub config: ConfigInfo,
    /// Summary statistics
    pub summary: SummaryStats,
    /// Detected clone groups
    pub clone_groups: Vec<CloneGroup>,
}

/// Tool metadata information
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Timestamp of analysis (ISO 8601)
    pub timestamp: String,
}

/// Configuration information for the analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    /// Detection mode used
    pub mode: String,
    /// Minimum tokens required
    pub min_tokens: usize,
    /// Minimum lines required
    pub min_lines: usize,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity_threshold: f64,
    /// Whether comments were ignored
    pub ignore_comments: bool,
    /// Whether whitespace was ignored
    pub ignore_whitespace: bool,
}

impl From<&DetectionConfig> for ConfigInfo {
    fn from(config: &DetectionConfig) -> Self {
        ConfigInfo {
            mode: format!("{:?}", config.mode),
            min_tokens: config.min_tokens,
            min_lines: config.min_lines,
            similarity_threshold: config.similarity_threshold,
            ignore_comments: config.ignore_comments,
            ignore_whitespace: config.ignore_whitespace,
        }
    }
}

/// Summary statistics for the analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryStats {
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Total number of clone instances found
    pub total_clone_instances: usize,
    /// Number of clone groups found
    pub clone_groups_count: usize,
    /// Estimated total duplicated lines
    pub duplicated_lines: usize,
    /// Duplication percentage (if calculable)
    pub duplication_percentage: Option<f64>,
    /// Analysis duration in seconds
    pub duration_seconds: f64,
}

impl JsonOutput {
    /// Create a new JSON output structure
    pub fn new(
        clone_groups: Vec<CloneGroup>,
        config: &DetectionConfig,
        files_count: usize,
        total_lines: usize,
        duration_seconds: f64,
    ) -> Self {
        let total_clone_instances: usize = clone_groups.iter().map(|g| g.size()).sum();
        let duplicated_lines: usize = clone_groups
            .iter()
            .map(|g| {
                g.instances
                    .iter()
                    .map(|c| c.end_line - c.start_line + 1)
                    .sum::<usize>()
            })
            .sum();

        let duplication_percentage = if total_lines > 0 {
            Some((duplicated_lines as f64 / total_lines as f64) * 100.0)
        } else {
            None
        };

        JsonOutput {
            metadata: ToolMetadata {
                name: "Deja-vu".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            config: ConfigInfo::from(config),
            summary: SummaryStats {
                files_analyzed: files_count,
                total_clone_instances,
                clone_groups_count: clone_groups.len(),
                duplicated_lines,
                duplication_percentage,
                duration_seconds,
            },
            clone_groups,
        }
    }

    /// Convert to pretty-printed JSON string
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
