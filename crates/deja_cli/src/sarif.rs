//! SARIF (Static Analysis Results Interchange Format) output
//!
//! Implements SARIF 2.1.0 specification for code duplication results
//! Reference: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html

use deja_core::{CloneGroup, DetectionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SARIF 2.1.0 report structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifReport {
    /// SARIF version (always "2.1.0")
    pub version: String,
    /// Schema URI
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Analysis runs
    pub runs: Vec<SarifRun>,
}

/// SARIF run (represents one analysis execution)
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRun {
    /// Tool information
    pub tool: SarifTool,
    /// Analysis results
    pub results: Vec<SarifResult>,
    /// Original file locations
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<SarifArtifact>,
}

/// Tool information
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifTool {
    /// Tool driver information
    pub driver: SarifDriver,
}

/// Tool driver (the analyzer itself)
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifDriver {
    /// Tool name
    pub name: String,
    /// Tool version
    pub version: String,
    /// Semantic version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_version: Option<String>,
    /// Information URI
    #[serde(rename = "informationUri", skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    /// Reporting rules/descriptors
    pub rules: Vec<SarifRule>,
}

/// Rule descriptor
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRule {
    /// Rule ID
    pub id: String,
    /// Short description
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
    /// Full description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    /// Help information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<SarifMessage>,
    /// Default severity level
    #[serde(rename = "defaultConfiguration", skip_serializing_if = "Option::is_none")]
    pub default_configuration: Option<SarifRuleConfiguration>,
}

/// Rule configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRuleConfiguration {
    /// Default severity level
    pub level: String,
}

/// Message with text
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifMessage {
    /// Message text
    pub text: String,
}

/// Analysis result (a detected issue)
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifResult {
    /// Rule ID that was violated
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    /// Result level (warning, error, note)
    pub level: String,
    /// Message describing the result
    pub message: SarifMessage,
    /// Locations where the issue was found
    pub locations: Vec<SarifLocation>,
    /// Related locations (other clone instances)
    #[serde(rename = "relatedLocations", skip_serializing_if = "Vec::is_empty")]
    pub related_locations: Vec<SarifLocation>,
    /// Additional properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Location in source code
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifLocation {
    /// Physical location in a file
    #[serde(rename = "physicalLocation")]
    pub physical_location: SarifPhysicalLocation,
    /// Optional message for this location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<SarifMessage>,
}

/// Physical location in a file
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    /// Artifact (file) location
    #[serde(rename = "artifactLocation")]
    pub artifact_location: SarifArtifactLocation,
    /// Region within the file
    pub region: SarifRegion,
}

/// Artifact (file) location
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    /// File URI or path
    pub uri: String,
    /// Optional URI base ID
    #[serde(rename = "uriBaseId", skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

/// Region within a file
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifRegion {
    /// Starting line (1-based)
    #[serde(rename = "startLine")]
    pub start_line: usize,
    /// Starting column (1-based)
    #[serde(rename = "startColumn", skip_serializing_if = "Option::is_none")]
    pub start_column: Option<usize>,
    /// Ending line (1-based)
    #[serde(rename = "endLine")]
    pub end_line: usize,
    /// Ending column (1-based)
    #[serde(rename = "endColumn", skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

/// Artifact (file) metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct SarifArtifact {
    /// Artifact location
    pub location: SarifArtifactLocation,
    /// Source language
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
}

impl SarifReport {
    /// Create a new SARIF report from clone groups
    pub fn from_clone_groups(
        clone_groups: Vec<CloneGroup>,
        _config: &DetectionConfig,
    ) -> Self {
        let mut results = Vec::new();

        for (group_idx, group) in clone_groups.iter().enumerate() {
            if group.instances.is_empty() {
                continue;
            }

            // First instance is the primary location
            let primary = &group.instances[0];

            // Rest are related locations
            let related: Vec<SarifLocation> = group.instances[1..]
                .iter()
                .map(|clone| SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: clone.file.display().to_string(),
                            uri_base_id: Some("%SRCROOT%".to_string()),
                        },
                        region: SarifRegion {
                            start_line: clone.start_line,
                            start_column: Some(clone.start_col + 1), // SARIF uses 1-based columns
                            end_line: clone.end_line,
                            end_column: Some(clone.end_col + 1),
                        },
                    },
                    message: Some(SarifMessage {
                        text: format!("Duplicate code instance {}", group_idx + 1),
                    }),
                })
                .collect();

            let mut properties = HashMap::new();
            properties.insert(
                "similarity".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(group.similarity * 100.0).unwrap(),
                ),
            );
            properties.insert(
                "clone_type".to_string(),
                serde_json::Value::String(group.clone_type.as_str().to_string()),
            );
            properties.insert(
                "instance_count".to_string(),
                serde_json::Value::Number(serde_json::Number::from(group.size())),
            );

            results.push(SarifResult {
                rule_id: format!("code-duplication/{}", group.clone_type.as_str()),
                level: "warning".to_string(),
                message: SarifMessage {
                    text: format!(
                        "Code duplication detected: {} with {:.1}% similarity ({} instances)",
                        group.clone_type.as_str(),
                        group.similarity * 100.0,
                        group.size()
                    ),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: primary.file.display().to_string(),
                            uri_base_id: Some("%SRCROOT%".to_string()),
                        },
                        region: SarifRegion {
                            start_line: primary.start_line,
                            start_column: Some(primary.start_col + 1),
                            end_line: primary.end_line,
                            end_column: Some(primary.end_col + 1),
                        },
                    },
                    message: None,
                }],
                related_locations: related,
                properties: Some(properties),
            });
        }

        SarifReport {
            version: "2.1.0".to_string(),
            schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "Deja-vu".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        semantic_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        information_uri: Some("https://github.com/stvnksslr/Deja-vu".to_string()),
                        rules: vec![
                            SarifRule {
                                id: "code-duplication/Type1".to_string(),
                                short_description: SarifMessage {
                                    text: "Type-1 clone: Exact duplicate code".to_string(),
                                },
                                full_description: Some(SarifMessage {
                                    text: "Identical code fragments except for variations in whitespace, layout, and comments.".to_string(),
                                }),
                                help: Some(SarifMessage {
                                    text: "Consider extracting the duplicated code into a shared function or module.".to_string(),
                                }),
                                default_configuration: Some(SarifRuleConfiguration {
                                    level: "warning".to_string(),
                                }),
                            },
                            SarifRule {
                                id: "code-duplication/Type2".to_string(),
                                short_description: SarifMessage {
                                    text: "Type-2 clone: Structurally identical code".to_string(),
                                },
                                full_description: Some(SarifMessage {
                                    text: "Structurally/syntactically identical fragments except for variations in identifiers, literals, types, layout, and comments.".to_string(),
                                }),
                                help: Some(SarifMessage {
                                    text: "Consider parameterizing the differences and extracting into a shared function.".to_string(),
                                }),
                                default_configuration: Some(SarifRuleConfiguration {
                                    level: "warning".to_string(),
                                }),
                            },
                            SarifRule {
                                id: "code-duplication/Type3".to_string(),
                                short_description: SarifMessage {
                                    text: "Type-3 clone: Similar code with modifications".to_string(),
                                },
                                full_description: Some(SarifMessage {
                                    text: "Copied fragments with further modifications such as changed, added, or removed statements.".to_string(),
                                }),
                                help: Some(SarifMessage {
                                    text: "Review for refactoring opportunities to reduce code duplication.".to_string(),
                                }),
                                default_configuration: Some(SarifRuleConfiguration {
                                    level: "note".to_string(),
                                }),
                            },
                        ],
                    },
                },
                results,
                artifacts: vec![],
            }],
        }
    }

    /// Convert to pretty-printed JSON string
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
