//! Utility functions for file collection and filtering

use crate::SourceFile;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Collect source files from given paths
pub fn collect_files(
    paths: &[PathBuf],
    extensions: &[&str],
    exclude_tests: bool,
) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if let Some(file) = collect_single_file(path, extensions, exclude_tests)? {
                files.push(file);
            }
        } else if path.is_dir() {
            collect_directory(path, extensions, exclude_tests, &mut files)?;
        } else {
            anyhow::bail!("Path does not exist: {}", path.display());
        }
    }

    Ok(files)
}

/// Collect a single file if it matches the extensions
fn collect_single_file(
    path: &Path,
    extensions: &[&str],
    exclude_tests: bool,
) -> Result<Option<SourceFile>> {
    if !should_include_file(path, extensions) {
        return Ok(None);
    }

    if exclude_tests && is_test_file(path) {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let language = detect_language(path);

    Ok(Some(SourceFile::new(path.to_path_buf(), content, language)))
}

/// Recursively collect files from a directory
fn collect_directory(
    dir: &Path,
    extensions: &[&str],
    exclude_tests: bool,
    files: &mut Vec<SourceFile>,
) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden files and directories
        if is_hidden(&path) {
            continue;
        }

        // Skip common build/dependency directories
        if should_skip_directory(&path) {
            continue;
        }

        if path.is_dir() {
            collect_directory(&path, extensions, exclude_tests, files)?;
        } else if path.is_file() {
            if let Some(file) = collect_single_file(&path, extensions, exclude_tests)? {
                files.push(file);
            }
        }
    }

    Ok(())
}

/// Check if a file should be included based on extensions
fn should_include_file(path: &Path, extensions: &[&str]) -> bool {
    if extensions.is_empty() {
        // If no extensions specified, include all files
        return true;
    }

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext))
        .unwrap_or(false)
}

/// Detect the language of a file based on its extension
fn detect_language(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext {
            "py" | "pyi" | "pyw" => "python",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "rs" => "rust",
            "java" => "java",
            "go" => "go",
            "c" => "c",
            "cpp" | "cc" | "cxx" => "cpp",
            "h" | "hpp" => "header",
            _ => "unknown",
        })
        .unwrap_or("unknown")
        .to_string()
}

/// Check if a path is hidden (starts with .)
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Check if a file is a test file (contains "test" in the name)
fn is_test_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_lowercase().contains("test"))
        .unwrap_or(false)
}

/// Check if a directory should be skipped
fn should_skip_directory(path: &Path) -> bool {
    const SKIP_DIRS: &[&str] = &[
        "node_modules",
        "target",
        "build",
        "dist",
        ".git",
        ".svn",
        ".hg",
        "__pycache__",
        ".pytest_cache",
        ".mypy_cache",
        "venv",
        "env",
        ".venv",
        ".env",
        "vendor",
        ".idea",
        ".vscode",
    ];

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| SKIP_DIRS.contains(&name))
        .unwrap_or(false)
}

/// Filter files by language
pub fn filter_by_language(files: Vec<SourceFile>, language: &str) -> Vec<SourceFile> {
    files
        .into_iter()
        .filter(|f| f.language == language)
        .collect()
}

/// Get statistics about collected files
pub fn file_statistics(files: &[SourceFile]) -> FileStats {
    let mut stats = FileStats::default();
    stats.total_files = files.len();

    for file in files {
        stats.total_bytes += file.content.len();
        stats.total_lines += file.content.lines().count();

        *stats.by_language.entry(file.language.clone()).or_insert(0) += 1;
    }

    stats
}

/// Statistics about collected files
#[derive(Debug, Default)]
pub struct FileStats {
    pub total_files: usize,
    pub total_bytes: usize,
    pub total_lines: usize,
    pub by_language: std::collections::HashMap<String, usize>,
}

impl FileStats {
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("{} files", self.total_files),
            format!("{} lines", self.total_lines),
            format!("{} bytes", self.total_bytes),
        ];

        if !self.by_language.is_empty() {
            let mut langs: Vec<_> = self.by_language.iter().collect();
            langs.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

            let lang_summary: Vec<String> = langs
                .iter()
                .map(|(lang, count)| format!("{}: {}", lang, count))
                .collect();

            parts.push(format!("Languages: {}", lang_summary.join(", ")));
        }

        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(Path::new("test.py")), "python");
        assert_eq!(detect_language(Path::new("test.js")), "javascript");
        assert_eq!(detect_language(Path::new("test.rs")), "rust");
        assert_eq!(detect_language(Path::new("test.unknown")), "unknown");
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".hidden")));
        assert!(!is_hidden(Path::new("visible")));
    }

    #[test]
    fn test_should_skip_directory() {
        assert!(should_skip_directory(Path::new("node_modules")));
        assert!(should_skip_directory(Path::new(".git")));
        assert!(should_skip_directory(Path::new("__pycache__")));
        assert!(!should_skip_directory(Path::new("src")));
    }

    #[test]
    fn test_should_include_file() {
        assert!(should_include_file(Path::new("test.py"), &["py"]));
        assert!(!should_include_file(Path::new("test.js"), &["py"]));
        assert!(should_include_file(Path::new("test.py"), &[])); // Empty = include all
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file(Path::new("test_example.py")));
        assert!(is_test_file(Path::new("example_test.py")));
        assert!(is_test_file(Path::new("TEST_CAPS.py")));
        assert!(is_test_file(Path::new("MyTestFile.py")));
        assert!(!is_test_file(Path::new("example.py")));
        assert!(!is_test_file(Path::new("main.py")));
    }

    #[test]
    fn test_file_stats_summary() {
        let mut stats = FileStats::default();
        stats.total_files = 10;
        stats.total_lines = 1000;
        stats.total_bytes = 50000;
        stats.by_language.insert("python".to_string(), 7);
        stats.by_language.insert("rust".to_string(), 3);

        let summary = stats.summary();
        assert!(summary.contains("10 files"));
        assert!(summary.contains("1000 lines"));
        assert!(summary.contains("python: 7"));
    }
}
