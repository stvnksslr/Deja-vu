//! Integration tests for output formats (JSON and SARIF)

use deja_core::{CloneType, Clone, CloneGroup, DetectionConfig, DetectionMode};
use deja_cli::output::JsonOutput;
use deja_cli::sarif::SarifReport;
use std::path::PathBuf;

#[test]
fn test_json_output_structure() {
    // Create sample clone groups
    let mut group = CloneGroup::new(CloneType::Type1, 1.0, "test-hash".to_string());
    group.add_instance(Clone::new(
        PathBuf::from("test1.py"),
        10,
        20,
        0,
        50,
        "test content".to_string(),
    ));
    group.add_instance(Clone::new(
        PathBuf::from("test2.py"),
        15,
        25,
        0,
        50,
        "test content".to_string(),
    ));

    let config = DetectionConfig {
        mode: DetectionMode::Balanced,
        min_tokens: 20,
        min_lines: 4,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let output = JsonOutput::new(
        vec![group],
        &config,
        2,    // files_count
        100,  // total_lines
        0.5,  // duration
    );

    // Verify structure
    assert_eq!(output.metadata.name, "Deja-vu");
    assert_eq!(output.config.min_tokens, 20);
    assert_eq!(output.config.min_lines, 4);
    assert_eq!(output.summary.files_analyzed, 2);
    assert_eq!(output.summary.clone_groups_count, 1);
    assert_eq!(output.summary.total_clone_instances, 2);
    assert_eq!(output.clone_groups.len(), 1);

    // Verify duplication percentage calculation
    assert!(output.summary.duplication_percentage.is_some());
    let dup_pct = output.summary.duplication_percentage.unwrap();
    assert!(dup_pct > 0.0 && dup_pct <= 100.0);

    // Test JSON serialization
    let json_str = output.to_json();
    assert!(json_str.is_ok());

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str.unwrap()).unwrap();
    assert!(parsed["metadata"].is_object());
    assert!(parsed["config"].is_object());
    assert!(parsed["summary"].is_object());
    assert!(parsed["clone_groups"].is_array());
}

#[test]
fn test_json_output_with_no_clones() {
    let config = DetectionConfig::default();
    let output = JsonOutput::new(
        vec![],
        &config,
        5,    // files_count
        500,  // total_lines
        1.0,  // duration
    );

    assert_eq!(output.summary.clone_groups_count, 0);
    assert_eq!(output.summary.total_clone_instances, 0);
    assert_eq!(output.summary.duplicated_lines, 0);
    assert_eq!(output.summary.duplication_percentage, Some(0.0));

    // Should still serialize successfully
    let json_str = output.to_json();
    assert!(json_str.is_ok());
}

#[test]
fn test_sarif_output_structure() {
    // Create sample clone groups
    let mut group = CloneGroup::new(CloneType::Type1, 1.0, "test-hash".to_string());
    group.add_instance(Clone::new(
        PathBuf::from("test1.py"),
        10,
        20,
        0,
        50,
        "test content".to_string(),
    ));
    group.add_instance(Clone::new(
        PathBuf::from("test2.py"),
        15,
        25,
        0,
        50,
        "test content".to_string(),
    ));

    let config = DetectionConfig::default();
    let report = SarifReport::from_clone_groups(vec![group], &config);

    // Verify SARIF structure
    assert_eq!(report.version, "2.1.0");
    assert_eq!(report.schema, "https://json.schemastore.org/sarif-2.1.0.json");
    assert_eq!(report.runs.len(), 1);

    let run = &report.runs[0];
    assert_eq!(run.tool.driver.name, "Deja-vu");
    assert!(!run.tool.driver.version.is_empty());
    assert_eq!(run.results.len(), 1);

    // Verify rule definitions
    assert_eq!(run.tool.driver.rules.len(), 3); // Type1, Type2, Type3
    assert_eq!(run.tool.driver.rules[0].id, "code-duplication/Type1");
    assert_eq!(run.tool.driver.rules[1].id, "code-duplication/Type2");
    assert_eq!(run.tool.driver.rules[2].id, "code-duplication/Type3");

    // Verify result structure
    let result = &run.results[0];
    assert_eq!(result.rule_id, "code-duplication/Type-1");
    assert_eq!(result.level, "warning");
    assert_eq!(result.locations.len(), 1);
    assert_eq!(result.related_locations.len(), 1);

    // Verify properties
    assert!(result.properties.is_some());
    let props = result.properties.as_ref().unwrap();
    assert!(props.contains_key("similarity"));
    assert!(props.contains_key("clone_type"));
    assert!(props.contains_key("instance_count"));

    // Test SARIF serialization
    let sarif_str = report.to_json();
    assert!(sarif_str.is_ok());

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str.unwrap()).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["runs"].is_array());
}

#[test]
fn test_sarif_output_with_no_clones() {
    let config = DetectionConfig::default();
    let report = SarifReport::from_clone_groups(vec![], &config);

    assert_eq!(report.runs[0].results.len(), 0);

    // Should still serialize successfully
    let sarif_str = report.to_json();
    assert!(sarif_str.is_ok());

    // Should still have valid SARIF structure
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str.unwrap()).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_sarif_location_format() {
    let mut group = CloneGroup::new(CloneType::Type2, 0.95, "test-hash".to_string());
    group.add_instance(Clone::new(
        PathBuf::from("src/main.py"),
        10,
        20,
        5,
        10,
        "code".to_string(),
    ));

    let config = DetectionConfig::default();
    let report = SarifReport::from_clone_groups(vec![group], &config);

    let result = &report.runs[0].results[0];
    let location = &result.locations[0];

    // Verify physical location structure
    assert_eq!(
        location.physical_location.artifact_location.uri,
        "src/main.py"
    );
    assert_eq!(
        location.physical_location.artifact_location.uri_base_id,
        Some("%SRCROOT%".to_string())
    );

    // Verify region (should be 1-based)
    assert_eq!(location.physical_location.region.start_line, 10);
    assert_eq!(location.physical_location.region.start_column, Some(6)); // 5 + 1
    assert_eq!(location.physical_location.region.end_line, 20);
    assert_eq!(location.physical_location.region.end_column, Some(11)); // 10 + 1
}

#[test]
fn test_json_summary_statistics() {
    // Create multiple clone groups to test aggregation
    let mut group1 = CloneGroup::new(CloneType::Type1, 1.0, "hash1".to_string());
    group1.add_instance(Clone::new(
        PathBuf::from("test1.py"),
        1,
        10,
        0,
        50,
        "content".to_string(),
    ));
    group1.add_instance(Clone::new(
        PathBuf::from("test2.py"),
        1,
        10,
        0,
        50,
        "content".to_string(),
    ));

    let mut group2 = CloneGroup::new(CloneType::Type2, 0.9, "hash2".to_string());
    group2.add_instance(Clone::new(
        PathBuf::from("test3.py"),
        5,
        15,
        0,
        50,
        "content".to_string(),
    ));
    group2.add_instance(Clone::new(
        PathBuf::from("test4.py"),
        5,
        15,
        0,
        50,
        "content".to_string(),
    ));
    group2.add_instance(Clone::new(
        PathBuf::from("test5.py"),
        5,
        15,
        0,
        50,
        "content".to_string(),
    ));

    let config = DetectionConfig::default();
    let output = JsonOutput::new(
        vec![group1, group2],
        &config,
        5,     // files
        1000,  // total lines
        2.5,   // duration
    );

    // Verify aggregated statistics
    assert_eq!(output.summary.clone_groups_count, 2);
    assert_eq!(output.summary.total_clone_instances, 5); // 2 + 3
    assert_eq!(output.summary.duplicated_lines, 53); // (10*2) + (11*3)
    assert_eq!(output.summary.files_analyzed, 5);
    assert_eq!(output.summary.duration_seconds, 2.5);

    // Verify duplication percentage
    let dup_pct = output.summary.duplication_percentage.unwrap();
    assert!((dup_pct - 5.3).abs() < 0.1); // 53/1000 * 100 = 5.3%
}
