//! Core data structures for representing code clones

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Types of code clones based on similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloneType {
    /// Type-1: Exact copies (ignoring whitespace and comments)
    Type1,
    /// Type-2: Syntactically identical (with renamed identifiers)
    Type2,
    /// Type-3: Copies with modifications (statements added/removed)
    Type3,
    /// Type-4: Semantically similar but syntactically different
    Type4,
}

impl CloneType {
    pub fn as_str(&self) -> &str {
        match self {
            CloneType::Type1 => "Type-1",
            CloneType::Type2 => "Type-2",
            CloneType::Type3 => "Type-3",
            CloneType::Type4 => "Type-4",
        }
    }
}

/// A single instance of a code clone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clone {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
    /// The actual code content
    pub content: String,
}

impl Clone {
    pub fn new(
        file: PathBuf,
        start_line: usize,
        end_line: usize,
        start_col: usize,
        end_col: usize,
        content: String,
    ) -> Self {
        Self {
            file,
            start_line,
            end_line,
            start_col,
            end_col,
            content,
        }
    }

    /// Returns the number of lines in this clone
    pub fn line_count(&self) -> usize {
        self.end_line - self.start_line + 1
    }
}

/// A group of related code clones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneGroup {
    pub clone_type: CloneType,
    pub instances: Vec<Clone>,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f64,
    /// Hash or signature for this clone group
    pub signature: String,
}

impl CloneGroup {
    pub fn new(clone_type: CloneType, similarity: f64, signature: String) -> Self {
        Self {
            clone_type,
            instances: Vec::new(),
            similarity,
            signature,
        }
    }

    pub fn add_instance(&mut self, clone: Clone) {
        self.instances.push(clone);
    }

    /// Returns the number of clone instances in this group
    pub fn size(&self) -> usize {
        self.instances.len()
    }

    /// Returns the average number of lines across all instances
    pub fn avg_lines(&self) -> f64 {
        if self.instances.is_empty() {
            return 0.0;
        }
        let total: usize = self.instances.iter().map(|c| c.line_count()).sum();
        total as f64 / self.instances.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_line_count() {
        let clone = Clone::new(
            PathBuf::from("test.py"),
            10,
            15,
            0,
            10,
            "code".to_string(),
        );
        assert_eq!(clone.line_count(), 6);
    }

    #[test]
    fn test_clone_group() {
        let mut group = CloneGroup::new(CloneType::Type1, 1.0, "hash123".to_string());

        group.add_instance(Clone::new(
            PathBuf::from("a.py"),
            1,
            5,
            0,
            10,
            "code".to_string(),
        ));

        group.add_instance(Clone::new(
            PathBuf::from("b.py"),
            10,
            14,
            0,
            10,
            "code".to_string(),
        ));

        assert_eq!(group.size(), 2);
        assert_eq!(group.avg_lines(), 5.0);
    }
}
