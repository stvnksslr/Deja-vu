"""
Deja-vu: An extremely fast code duplication detector

This package provides a Python interface to the Deja-vu code duplication
detector, which is written in Rust for performance.
"""

from deja_vu.deja_vu import (
    Clone,
    CloneGroup,
    CloneType,
    DetectionConfig,
    detect_clones,
    version,
)

__all__ = [
    "Clone",
    "CloneGroup",
    "CloneType",
    "DetectionConfig",
    "detect_clones",
    "version",
]

__version__ = version()
