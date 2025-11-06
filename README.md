# Deja-vu 🔍

An extremely fast code duplication detector, inspired by [Ruff](https://github.com/astral-sh/ruff).

> **Status**: 🚧 Early Development - Core architecture in place, detection algorithms in progress

## Overview

Deja-vu is a high-performance code duplication detection tool written in Rust with Python bindings. It's designed to be:

- **Fast**: Written in Rust for maximum performance
- **Extensible**: Plugin architecture for multiple programming languages
- **Easy to use**: Simple CLI and Python API
- **Accurate**: Multiple detection modes from fast to precise

## Features

### Detection Modes

- **Fast**: Token-based detection for Type-1 and Type-2 clones
- **Balanced**: AST-based detection for Type-1, Type-2, and Type-3 clones
- **Precise**: Graph-based detection including Type-4 clones (planned)

### Clone Types

- **Type-1**: Exact copies (ignoring whitespace and comments)
- **Type-2**: Syntactically identical (with renamed identifiers)
- **Type-3**: Copies with modifications (statements added/removed)
- **Type-4**: Semantically similar but syntactically different (planned)

### Language Support

- ✅ Python (via Ruff's Python parser)
- 🔜 JavaScript/TypeScript (planned)
- 🔜 Rust (planned)
- 🔜 Java (planned)
- 🔜 Go (planned)

## Installation

### From Source (Rust)

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/stvnksslr/Deja-vu.git
cd Deja-vu
cargo build --release

# The binary will be at target/release/deja
```

### Python Package (Coming Soon)

```bash
pip install deja-vu
```

## Usage

### CLI

```bash
# Basic usage
deja check /path/to/code

# With options
deja check /path/to/code \
  --mode balanced \
  --min-lines 5 \
  --min-tokens 50 \
  --threshold 0.85

# Check multiple paths
deja check src/ tests/ --verbose

# Version information
deja version
```

### Python API

```python
import deja_vu

# Configure detection
config = deja_vu.DetectionConfig(
    mode="balanced",
    min_lines=5,
    min_tokens=50,
    similarity_threshold=0.85
)

# Or use presets
config = deja_vu.DetectionConfig.balanced()

# Detect clones
clones = deja_vu.detect_clones(
    files=["path/to/file1.py", "path/to/file2.py"],
    config=config
)

# Process results
for group in clones:
    print(f"Clone Type: {group.clone_type}")
    print(f"Similarity: {group.similarity:.2%}")
    print(f"Instances: {group.size}")

    for instance in group.instances:
        print(f"  {instance.file}:{instance.start_line}-{instance.end_line}")
```

## Architecture

Deja-vu follows a modular architecture inspired by Ruff:

```
deja-vu/
├── crates/
│   ├── deja_core/       # Core detection algorithms
│   ├── deja_ast/        # Language-agnostic AST representation
│   ├── deja_python/     # Python language support
│   ├── deja_cli/        # CLI application
│   └── deja_py/         # Python bindings (PyO3)
├── python/
│   └── deja_vu/         # Python package
└── docs/                # Documentation
```

### Core Components

- **deja_core**: Core detection algorithms, similarity metrics, and data structures
- **deja_ast**: Generic AST representation for language-agnostic processing
- **deja_python**: Python-specific parser and tokenizer using Ruff's parser
- **deja_cli**: Command-line interface
- **deja_py**: Python bindings using PyO3

## Development

### Prerequisites

- Rust 1.75+ (specified in `rust-toolchain.toml`)
- Python 3.8+ (for Python bindings)

### Build

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Build Python package
pip install maturin
maturin develop
```

### Test

```bash
# Run Rust tests
cargo test

# Run with coverage
cargo test --all-features

# Run Python tests
pytest tests/
```

### Lint & Format

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features
```

## Roadmap

### Phase 1: MVP (Current)
- [x] Project structure and architecture
- [x] Core abstractions (AST, tokens, clones)
- [x] Python language support foundation
- [x] CLI skeleton
- [x] Python bindings skeleton
- [ ] Token-based detection (Fast mode)
- [ ] Basic reporting

### Phase 2: Enhanced Detection
- [ ] AST-based detection (Balanced mode)
- [ ] Similarity metrics refinement
- [ ] Configuration file support (deja.toml)
- [ ] JSON/SARIF output formats

### Phase 3: Multi-Language
- [ ] Tree-sitter integration
- [ ] JavaScript/TypeScript support
- [ ] Rust support
- [ ] Java support

### Phase 4: Advanced Features
- [ ] Graph-based detection (Precise mode)
- [ ] IDE integrations (LSP)
- [ ] Auto-fix suggestions
- [ ] Incremental analysis

## Benchmarks

Coming soon - we'll compare against:
- PMD CPD
- SonarQube
- Simian
- CloneDR

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by [Ruff](https://github.com/astral-sh/ruff) - an amazing example of Rust + Python
- Uses Ruff's Python parser for Python language support
- Research based on academic work in code clone detection

## Citations

If you use Deja-vu in academic research, please cite:

```bibtex
@software{deja_vu,
  title = {Deja-vu: Fast Code Duplication Detection},
  author = {Deja-vu Contributors},
  year = {2024},
  url = {https://github.com/stvnksslr/Deja-vu}
}
```

---

**Note**: This project is in early development. APIs and features are subject to change.
