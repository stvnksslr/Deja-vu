# Contributing to Deja-vu

Thank you for your interest in contributing to Deja-vu! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Python 3.8 or later (for Python bindings)
- Git

### Setting Up Development Environment

1. **Fork and clone the repository**

```bash
git clone https://github.com/YOUR_USERNAME/Deja-vu.git
cd Deja-vu
```

2. **Install Rust**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. **Install Python dependencies** (optional, for Python development)

```bash
pip install maturin pytest black ruff
```

4. **Build the project**

```bash
cargo build
```

5. **Run tests**

```bash
cargo test
```

## Development Workflow

### Making Changes

1. **Create a branch** for your feature or bugfix

```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes** following our coding standards

3. **Write tests** for your changes

4. **Run tests** to ensure everything passes

```bash
cargo test
cargo clippy
cargo fmt --check
```

5. **Commit your changes** with a clear message

```bash
git commit -m "Add feature: your feature description"
```

### Code Style

- **Rust**: Follow the standard Rust style guide
  - Run `cargo fmt` before committing
  - Run `cargo clippy` and address warnings
  - Use meaningful variable names
  - Add doc comments for public APIs

- **Python**: Follow PEP 8
  - Use Black for formatting
  - Use Ruff for linting
  - Type hints where appropriate

### Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Add integration tests for complex features
- Aim for high code coverage

### Documentation

- Update README.md if adding user-facing features
- Add inline documentation for complex code
- Update CHANGELOG.md (we'll add this)

## Pull Request Process

1. **Update documentation** as needed
2. **Ensure all tests pass**
3. **Rebase your branch** on the latest main
4. **Create a pull request** with:
   - Clear title describing the change
   - Description of what and why
   - Link to any related issues
5. **Wait for review** and address feedback

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Expected Behavior

- Be respectful and considerate
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Accept criticism gracefully

### Unacceptable Behavior

- Harassment or discriminatory language
- Trolling or insulting comments
- Personal attacks
- Publishing others' private information

## Project Structure

```
deja-vu/
├── crates/
│   ├── deja_core/       # Core algorithms
│   ├── deja_ast/        # AST representations
│   ├── deja_python/     # Python support
│   ├── deja_cli/        # CLI
│   └── deja_py/         # Python bindings
├── python/              # Python package
├── tests/               # Integration tests
└── docs/                # Documentation
```

## Areas for Contribution

### Beginner-Friendly

- Documentation improvements
- Adding tests
- Bug fixes
- Example code

### Intermediate

- New language support
- Performance optimizations
- CLI improvements
- Output formatters

### Advanced

- Core detection algorithms
- Graph-based analysis
- Parser development
- Architecture improvements

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in issues
- Ask questions in pull request comments

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in academic citations if applicable

Thank you for contributing to Deja-vu! 🎉
