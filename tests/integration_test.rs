//! Integration tests for Deja-vu clone detector
//!
//! These tests verify end-to-end functionality using real Python files.

use deja_core::{
    collect_files, CloneDetector, DetectionConfig, DetectionMode, TokenBasedDetector,
};
use deja_python::PythonTokenizer;
use std::path::PathBuf;

fn setup_detector() -> TokenBasedDetector {
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));
    detector
}

#[test]
fn test_integration_simple_duplicate() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 5,
        min_lines: 2,
        similarity_threshold: 0.8,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/simple_duplicate.py");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    assert_eq!(files.len(), 1, "Should find one Python file");

    let result = detector.detect(&files, &config);
    assert!(result.is_ok(), "Detection should succeed");

    let groups = result.unwrap();
    // The fixture has two similar functions (add and sum_values)
    // They should be detected as clones
    assert!(
        groups.len() > 0,
        "Should detect clones in simple_duplicate.py"
    );
}

#[test]
fn test_integration_no_duplicates() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 5,
        min_lines: 2,
        similarity_threshold: 0.8,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/no_duplicates.py");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    assert_eq!(files.len(), 1);

    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    let groups = result.unwrap();
    // Should have no clones or very few
    assert!(
        groups.len() < 2,
        "Should have minimal or no clones in unique code"
    );
}

#[test]
fn test_integration_multiple_clones() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 8,
        min_lines: 3,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/multiple_clones.py");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    let groups = result.unwrap();
    // Should detect multiple clone groups
    assert!(
        groups.len() >= 2,
        "Should detect multiple clone groups in multiple_clones.py"
    );

    // Verify clone groups have correct structure
    for group in &groups {
        assert!(
            group.size() >= 2,
            "Each clone group should have at least 2 instances"
        );
        assert!(
            group.similarity > 0.0 && group.similarity <= 1.0,
            "Similarity should be between 0 and 1"
        );
    }
}

#[test]
fn test_integration_respects_min_tokens() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 100, // Very high threshold
        min_lines: 1,
        similarity_threshold: 0.8,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    let groups = result.unwrap();
    // With very high min_tokens, should find no clones
    assert_eq!(
        groups.len(),
        0,
        "Should find no clones with very high min_tokens threshold"
    );
}

#[test]
fn test_integration_too_short_file() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 10,
        min_lines: 3,
        similarity_threshold: 0.8,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/too_short.py");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    let groups = result.unwrap();
    // File is too short to trigger detection
    assert_eq!(groups.len(), 0, "Should not detect clones in very short file");
}

#[test]
fn test_integration_examples_directory() {
    let detector = setup_detector();
    let config = DetectionConfig {
        mode: DetectionMode::Fast,
        min_tokens: 10,
        min_lines: 3,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    let examples_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/python_duplicates/");

    let files = collect_files(&[examples_path], &["py"], false).expect("Failed to collect files");
    assert!(files.len() >= 2, "Should find multiple Python example files");

    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    let groups = result.unwrap();
    // Examples directory is designed to have duplicates
    assert!(
        groups.len() > 0,
        "Should detect clones in examples directory"
    );

    // Verify all groups are properly formed
    for group in &groups {
        assert!(group.size() >= 2);
        assert!(!group.instances.is_empty());

        for instance in &group.instances {
            assert!(instance.start_line > 0);
            assert!(instance.end_line >= instance.start_line);
            assert!(!instance.content.is_empty());
        }
    }
}

#[test]
fn test_integration_multiple_files() {
    let detector = setup_detector();
    let config = DetectionConfig::default();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");
    assert!(files.len() >= 3, "Should collect multiple test fixture files");

    let result = detector.detect(&files, &config);
    assert!(result.is_ok());

    // Just verify it completes successfully
    let _ = result.unwrap();
}

#[test]
fn test_integration_empty_directory() {
    let detector = setup_detector();
    let config = DetectionConfig::default();

    // Try to collect from a non-existent directory
    let nonexistent_path = PathBuf::from("/nonexistent/path/to/files");

    let result = collect_files(&[nonexistent_path], &["py"], false);
    assert!(result.is_err(), "Should fail for non-existent directory");
}

#[test]
fn test_integration_config_modes() {
    let detector = setup_detector();
    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/multiple_clones.py");

    let files = collect_files(&[fixture_path], &["py"], false).expect("Failed to collect files");

    // Test Fast mode
    let fast_config = DetectionConfig::fast();
    let result = detector.detect(&files, &fast_config);
    assert!(result.is_ok());

    // Test Balanced mode
    let balanced_config = DetectionConfig::balanced();
    let result = detector.detect(&files, &balanced_config);
    assert!(result.is_ok());

    // All modes should work (even if Balanced/Precise use same algorithm for now)
    let precise_config = DetectionConfig::precise();
    let result = detector.detect(&files, &precise_config);
    assert!(result.is_ok());
}
