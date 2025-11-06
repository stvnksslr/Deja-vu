//! Integration test for docstring filtering in clone detection

use deja_core::{CloneDetector, DetectionConfig, DetectionMode, TokenBasedDetector};
use deja_python::PythonTokenizer;
use std::path::PathBuf;

#[test]
fn test_docstrings_excluded_from_duplicate_display() {
    // Create detector with Python tokenizer
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));

    // Create test files with identical code but different docstrings
    let test_code1 = r#"
def calculate(x):
    """First docstring - this should be ignored"""
    result = x * 2
    return result
"#;

    let test_code2 = r#"
def calculate(x):
    """Completely different docstring - also ignored"""
    result = x * 2
    return result
"#;

    // Create source files
    let files = vec![
        deja_core::SourceFile::new(
            PathBuf::from("test1.py"),
            test_code1.to_string(),
            "python".to_string(),
        ),
        deja_core::SourceFile::new(
            PathBuf::from("test2.py"),
            test_code2.to_string(),
            "python".to_string(),
        ),
    ];

    // Configure detection with comments ignored (default)
    let config = DetectionConfig {
        mode: DetectionMode::Balanced,
        min_lines: 2,
        min_tokens: 5,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    // Detect clones
    let clone_groups = detector.detect(&files, &config).unwrap();

    // Should detect a duplicate (same code, different docstrings)
    assert!(
        !clone_groups.is_empty(),
        "Should detect duplicate despite different docstrings"
    );

    let group = &clone_groups[0];

    assert_eq!(
        group.instances.len(),
        2,
        "Should have 2 instances of the duplicate"
    );

    // Verify that the displayed content does NOT contain docstrings
    for instance in &group.instances {
        let content = &instance.content;

        // Should NOT contain the docstrings
        assert!(
            !content.contains("First docstring"),
            "Content should not include first docstring"
        );
        assert!(
            !content.contains("Completely different"),
            "Content should not include second docstring"
        );

        // SHOULD contain the actual code
        assert!(
            content.contains("result") && content.contains("x") && content.contains("2"),
            "Content should include the actual duplicate code"
        );
    }
}

#[test]
fn test_same_docstring_different_code_not_duplicate() {
    // Create detector with Python tokenizer
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));

    // Create test files with same docstring but significantly different code
    let test_code1 = r#"
def process_data(items):
    """Process the input data"""
    total = 0
    for item in items:
        total += item * 2
    return total
"#;

    let test_code2 = r#"
def process_data(items):
    """Process the input data"""
    results = []
    for item in items:
        results.append(item + 10)
    return results
"#;

    // Create source files
    let files = vec![
        deja_core::SourceFile::new(
            PathBuf::from("test1.py"),
            test_code1.to_string(),
            "python".to_string(),
        ),
        deja_core::SourceFile::new(
            PathBuf::from("test2.py"),
            test_code2.to_string(),
            "python".to_string(),
        ),
    ];

    // Configure detection
    let config = DetectionConfig {
        mode: DetectionMode::Balanced,
        min_lines: 3,
        min_tokens: 10,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    // Detect clones
    let clone_groups = detector.detect(&files, &config).unwrap();

    // Should NOT detect a duplicate (different code logic, same docstring)
    assert!(
        clone_groups.is_empty(),
        "Should not detect duplicate when only docstrings match but code is different"
    );
}

#[test]
fn test_class_docstrings_excluded() {
    // Create detector with Python tokenizer
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));

    // Create test files with identical classes but different docstrings
    let test_code1 = r#"
class Calculator:
    """First calculator class with detailed documentation"""

    def add(self, a, b):
        return a + b

    def subtract(self, a, b):
        return a - b
"#;

    let test_code2 = r#"
class Calculator:
    """Second calculator class with different docs"""

    def add(self, a, b):
        return a + b

    def subtract(self, a, b):
        return a - b
"#;

    // Create source files
    let files = vec![
        deja_core::SourceFile::new(
            PathBuf::from("test1.py"),
            test_code1.to_string(),
            "python".to_string(),
        ),
        deja_core::SourceFile::new(
            PathBuf::from("test2.py"),
            test_code2.to_string(),
            "python".to_string(),
        ),
    ];

    // Configure detection
    let config = DetectionConfig {
        mode: DetectionMode::Balanced,
        min_lines: 4,
        min_tokens: 10,
        similarity_threshold: 0.85,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    // Detect clones
    let clone_groups = detector.detect(&files, &config).unwrap();

    // Should detect a duplicate
    assert!(
        !clone_groups.is_empty(),
        "Should detect duplicate class despite different docstrings"
    );

    // Verify that class docstrings are not in the displayed content
    for group in &clone_groups {
        for instance in &group.instances {
            let content = &instance.content;

            // Should NOT contain the class docstrings
            assert!(
                !content.contains("detailed documentation")
                    && !content.contains("different docs"),
                "Content should not include class docstrings"
            );

            // SHOULD contain the actual code
            assert!(
                content.contains("add") || content.contains("subtract"),
                "Content should include the actual method names"
            );
        }
    }
}
