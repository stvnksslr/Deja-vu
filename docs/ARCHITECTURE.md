# Deja-vu Architecture

This document describes the architecture and design decisions of Deja-vu.

## Overview

Deja-vu is designed as a modular, high-performance code duplication detector with a Rust core and Python bindings, inspired by Ruff's architecture.

## Design Principles

1. **Performance First**: Core algorithms written in Rust for maximum speed
2. **Language Agnostic**: Generic AST representation allows supporting multiple languages
3. **Extensible**: Plugin architecture for language-specific parsers
4. **User Friendly**: Simple CLI and Python API
5. **Accurate**: Multiple detection modes balancing speed vs accuracy

## Architecture Layers

### Layer 1: Core Abstractions (`deja_core`)

The foundation layer providing language-agnostic data structures and algorithms:

- **Clone Types**: Representation of Type-1 through Type-4 clones
- **Detection Config**: Configuration for detection algorithms
- **Similarity Metrics**: Various algorithms for comparing code fragments
- **Token Representation**: Generic token types for lexical analysis
- **Hash Functions**: Fast hashing for clone detection

**Key Design Decisions:**
- Use traits for extensibility (`CloneDetector`, `SimilarityMetric`)
- Generic structures to support multiple languages
- Performance-critical code using Rust's zero-cost abstractions

### Layer 2: AST Representation (`deja_ast`)

Generic AST representation that can be produced from any language:

- **NodeKind**: Language-agnostic node types (Function, Class, Loop, etc.)
- **AstNode**: Generic node structure with span information
- **Ast**: Complete tree representation with metadata
- **ToGenericAst**: Trait for converting language-specific ASTs

**Key Design Decisions:**
- Trade some language specificity for uniformity
- Include span information for accurate location reporting
- Keep structure simple for easy traversal and comparison

### Layer 3: Language Support (`deja_python`, etc.)

Language-specific parsers and tokenizers:

#### Python Support (`deja_python`)
- **Parser**: Uses Ruff's Python parser for AST generation
- **Tokenizer**: Lexical analysis for token-based detection
- **Conversion**: Maps Python AST to generic AST

**Key Design Decisions:**
- Leverage existing, battle-tested parsers (Ruff for Python)
- Provide both token and AST interfaces
- Handle syntax errors gracefully

#### Future Languages
- Will follow similar pattern with appropriate parser libraries
- Tree-sitter integration for broad language support

### Layer 4: Detection Engine (In Progress)

Implements the actual clone detection algorithms:

#### Token-Based Detection (Fast Mode)
```rust
1. Tokenize all source files
2. Normalize tokens (identifiers → $ID, literals → $LIT)
3. Generate rolling hash over token windows
4. Find matches using hash table
5. Extend matches to find full clone region
6. Filter by minimum size thresholds
```

**Complexity**: O(n) where n is total tokens
**Detects**: Type-1, Type-2 clones

#### AST-Based Detection (Balanced Mode)
```rust
1. Parse files to generic AST
2. Generate subtree hashes
3. Find candidate matches
4. Compare subtrees using tree edit distance
5. Calculate similarity score
6. Group related clones
```

**Complexity**: O(n²) worst case, O(n log n) typical
**Detects**: Type-1, Type-2, Type-3 clones

#### Graph-Based Detection (Precise Mode - Future)
```rust
1. Build Program Dependency Graphs (PDG)
2. Extract subgraphs
3. Compute graph isomorphism candidates
4. Verify semantic equivalence
5. Rank by confidence
```

**Complexity**: O(n³) worst case
**Detects**: All clone types including Type-4

### Layer 5: User Interfaces

#### CLI (`deja_cli`)
- Command-line interface using Clap
- Colorized output
- Multiple output formats (text, JSON, SARIF)
- Progress reporting for large codebases

#### Python Bindings (`deja_py`)
- PyO3 for Rust-Python bridge
- Pythonic API with proper error handling
- Type stubs for IDE support
- Minimal overhead wrapper around Rust core

## Data Flow

```
┌─────────────────┐
│  Source Files   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Language       │
│  Parser         │
│  (deja_python)  │
└────────┬────────┘
         │
         ├─────────────┐
         v             v
┌────────────┐   ┌────────────┐
│  Tokens    │   │  AST       │
└─────┬──────┘   └─────┬──────┘
      │                │
      v                v
┌─────────────────────────┐
│  Generic               │
│  Representation        │
│  (deja_ast,            │
│   deja_core)           │
└───────────┬─────────────┘
            │
            v
┌─────────────────────────┐
│  Detection Engine       │
│  - Hash computation     │
│  - Similarity analysis  │
│  - Clone grouping       │
└───────────┬─────────────┘
            │
            v
┌─────────────────────────┐
│  Results                │
│  (CloneGroup[])         │
└───────────┬─────────────┘
            │
            ├─────────────┐
            v             v
      ┌─────────┐   ┌─────────┐
      │   CLI   │   │ Python  │
      │ Output  │   │   API   │
      └─────────┘   └─────────┘
```

## Performance Considerations

### Parallelization
- File parsing done in parallel using Rayon
- Clone detection parallelized per file pair
- Lock-free data structures (DashMap) for concurrent access

### Memory Management
- Streaming processing for large codebases
- Token sequences stored efficiently
- AST nodes deduplicated where possible

### Caching
- Parse results cached across detection modes
- Hash values memoized
- Configuration-dependent cache invalidation

## Extensibility Points

1. **Language Plugins**: Implement `LanguageParser` trait
2. **Detection Algorithms**: Implement `CloneDetector` trait
3. **Similarity Metrics**: Implement `SimilarityMetric` trait
4. **Output Formatters**: Add new format handlers in CLI
5. **Filters**: Custom post-processing of results

## Configuration

Future: `deja.toml` file following Ruff's model:

```toml
[detection]
mode = "balanced"
min-lines = 5
min-tokens = 50
threshold = 0.85

[languages]
python = { enabled = true }
javascript = { enabled = true }

[exclude]
paths = ["vendor/", "node_modules/"]

[report]
format = "text"
output = "clones.txt"
```

## Testing Strategy

1. **Unit Tests**: Each module extensively tested
2. **Integration Tests**: End-to-end detection scenarios
3. **Benchmark Tests**: Performance regression detection
4. **Language Tests**: Parser correctness for each language
5. **Comparison Tests**: Validate against known clone datasets

## Future Enhancements

1. **Incremental Analysis**: Only analyze changed files
2. **IDE Integration**: Language Server Protocol (LSP)
3. **Machine Learning**: Learn project-specific patterns
4. **Auto-Refactoring**: Suggest deduplication strategies
5. **Visualization**: Web-based clone visualization

## References

- [Ruff Architecture](https://github.com/astral-sh/ruff)
- ["Comparison and Evaluation of Clone Detection Tools"](https://www.researchgate.net/)
- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- [PyO3 Guide](https://pyo3.rs/)
