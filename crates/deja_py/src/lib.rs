//! Python bindings for Deja-vu using PyO3
//!
//! This crate provides Python bindings for the Deja-vu code duplication detector,
//! allowing it to be used as a Python library.

use deja_core::clone::{Clone, CloneGroup, CloneType};
use deja_core::detector::{CloneDetector, DetectionConfig, DetectionMode};
use deja_core::{collect_files, TokenBasedDetector};
use deja_python::PythonTokenizer;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::path::PathBuf;

/// A Python module implemented in Rust.
#[pymodule]
fn deja_vu(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDetectionConfig>()?;
    m.add_class::<PyCloneType>()?;
    m.add_class::<PyClone>()?;
    m.add_class::<PyCloneGroup>()?;
    m.add_function(wrap_pyfunction!(detect_clones, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

/// Detection configuration for clone detection
#[pyclass(name = "DetectionConfig")]
#[derive(Clone)]
struct PyDetectionConfig {
    config: DetectionConfig,
}

#[pymethods]
impl PyDetectionConfig {
    #[new]
    #[pyo3(signature = (mode="balanced", min_tokens=50, min_lines=5, similarity_threshold=0.85, ignore_comments=true, ignore_whitespace=true))]
    fn new(
        mode: &str,
        min_tokens: usize,
        min_lines: usize,
        similarity_threshold: f64,
        ignore_comments: bool,
        ignore_whitespace: bool,
    ) -> PyResult<Self> {
        let detection_mode = match mode {
            "fast" => DetectionMode::Fast,
            "balanced" => DetectionMode::Balanced,
            "precise" => DetectionMode::Precise,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Invalid mode '{}'. Must be 'fast', 'balanced', or 'precise'",
                    mode
                )))
            }
        };

        Ok(Self {
            config: DetectionConfig {
                mode: detection_mode,
                min_tokens,
                min_lines,
                similarity_threshold,
                ignore_comments,
                ignore_whitespace,
            },
        })
    }

    #[staticmethod]
    fn fast() -> Self {
        Self {
            config: DetectionConfig::fast(),
        }
    }

    #[staticmethod]
    fn balanced() -> Self {
        Self {
            config: DetectionConfig::balanced(),
        }
    }

    #[staticmethod]
    fn precise() -> Self {
        Self {
            config: DetectionConfig::precise(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "DetectionConfig(mode={:?}, min_tokens={}, min_lines={}, threshold={})",
            self.config.mode,
            self.config.min_tokens,
            self.config.min_lines,
            self.config.similarity_threshold
        )
    }
}

/// Type of code clone
#[pyclass(name = "CloneType")]
#[derive(Clone)]
struct PyCloneType {
    clone_type: CloneType,
}

#[pymethods]
impl PyCloneType {
    fn __repr__(&self) -> String {
        self.clone_type.as_str().to_string()
    }

    fn __str__(&self) -> String {
        self.clone_type.as_str().to_string()
    }
}

/// A single instance of a code clone
#[pyclass(name = "Clone")]
#[derive(Clone)]
struct PyClone {
    clone: Clone,
}

#[pymethods]
impl PyClone {
    #[getter]
    fn file(&self) -> String {
        self.clone.file.display().to_string()
    }

    #[getter]
    fn start_line(&self) -> usize {
        self.clone.start_line
    }

    #[getter]
    fn end_line(&self) -> usize {
        self.clone.end_line
    }

    #[getter]
    fn start_col(&self) -> usize {
        self.clone.start_col
    }

    #[getter]
    fn end_col(&self) -> usize {
        self.clone.end_col
    }

    #[getter]
    fn content(&self) -> String {
        self.clone.content.clone()
    }

    #[getter]
    fn line_count(&self) -> usize {
        self.clone.line_count()
    }

    fn __repr__(&self) -> String {
        format!(
            "Clone(file='{}', lines={}-{}, {} lines)",
            self.file(),
            self.start_line(),
            self.end_line(),
            self.line_count()
        )
    }
}

/// A group of related code clones
#[pyclass(name = "CloneGroup")]
#[derive(Clone)]
struct PyCloneGroup {
    group: CloneGroup,
}

#[pymethods]
impl PyCloneGroup {
    #[getter]
    fn clone_type(&self) -> String {
        self.group.clone_type.as_str().to_string()
    }

    #[getter]
    fn similarity(&self) -> f64 {
        self.group.similarity
    }

    #[getter]
    fn instances(&self) -> Vec<PyClone> {
        self.group
            .instances
            .iter()
            .map(|c| PyClone { clone: c.clone() })
            .collect()
    }

    #[getter]
    fn size(&self) -> usize {
        self.group.size()
    }

    #[getter]
    fn avg_lines(&self) -> f64 {
        self.group.avg_lines()
    }

    fn __repr__(&self) -> String {
        format!(
            "CloneGroup(type={}, instances={}, similarity={:.2})",
            self.clone_type(),
            self.size(),
            self.similarity()
        )
    }

    fn __len__(&self) -> usize {
        self.size()
    }
}

/// Detect code clones in the given files
#[pyfunction]
#[pyo3(signature = (files, config=None))]
fn detect_clones(files: Vec<String>, config: Option<PyDetectionConfig>) -> PyResult<Vec<PyCloneGroup>> {
    let config = config.unwrap_or_else(|| PyDetectionConfig {
        config: DetectionConfig::default(),
    });

    // Convert string paths to PathBuf
    let paths: Vec<PathBuf> = files.iter().map(|f| PathBuf::from(f)).collect();

    // Collect Python files (don't exclude tests by default for Python API)
    let source_files = collect_files(&paths, &["py"], false)
        .map_err(|e| PyIOError::new_err(format!("Failed to collect files: {}", e)))?;

    // Create detector with Python tokenizer
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));

    // Run detection
    let clone_groups = detector
        .detect(&source_files, &config.config)
        .map_err(|e| PyValueError::new_err(format!("Detection failed: {}", e)))?;

    // Convert to Python types
    let py_groups: Vec<PyCloneGroup> = clone_groups.into_iter().map(|g| g.into()).collect();

    Ok(py_groups)
}

/// Get the version of Deja-vu
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl From<Clone> for PyClone {
    fn from(clone: Clone) -> Self {
        Self { clone }
    }
}

impl From<CloneGroup> for PyCloneGroup {
    fn from(group: CloneGroup) -> Self {
        Self { group }
    }
}
