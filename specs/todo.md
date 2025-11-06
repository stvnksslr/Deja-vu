# Deja-vu Development Roadmap

**Status**: Phase 1 (Token Detection + Basic Reporting) - 85% Complete
**Last Updated**: 2025-11-06

---

## Phase 1: Token-Based Detection + Basic Reporting (FAST MODE)

**Goal**: Complete token-based clone detection with CLI reporting
**Target**: Basic working tool for Type-1 and Type-2 clone detection

### Phase 1.1: Token Detection Fixes ⚠️ HIGH PRIORITY

#### 1.1.1 Fix Python Tokenizer Line Numbers 🔴 CRITICAL
**File**: `crates/deja_python/src/tokenizer.rs:24`
**Issue**: Using approximation `range.start().to_usize() / 1000` for line numbers
**Impact**: Incorrect line/column reporting in output

**Tasks**:
- [ ] Research RustPython `TextRange` API to get proper line numbers
- [ ] Implement correct line/column extraction from source offset
- [ ] Add test cases verifying line numbers match source file
- [ ] Update tokenizer to maintain line/column state properly

**Acceptance Criteria**:
- [ ] Line numbers in output match actual source file line numbers
- [ ] Column numbers accurately reflect token positions
- [ ] Tests verify correctness with multi-line Python code

#### 1.1.2 Fix Detection Threshold Calibration 🟡 MEDIUM
**Issue**: Test fixtures with obvious clones aren't being detected
**Files**:
- `tests/fixtures/simple_duplicate.py` - Type-2 clones not detected
- `tests/fixtures/multiple_clones.py` - Should find 3 groups

**Tasks**:
- [ ] Run detector with very low thresholds to verify it CAN detect clones
- [ ] Analyze why boilerplate filtering may be too aggressive
- [ ] Adjust default thresholds for better sensitivity
- [ ] Add debug logging to show what's being filtered out
- [ ] Document threshold tuning guide

**Acceptance Criteria**:
- [ ] `simple_duplicate.py` detects the add/sum_values functions as clones
- [ ] `multiple_clones.py` detects 2-3 clone groups
- [ ] Integration tests pass without requiring extreme thresholds

#### 1.1.3 Code Cleanup 🟢 LOW PRIORITY
**Tasks**:
- [ ] Remove unused `token_count` field from `CloneCandidate` struct
- [ ] Remove unused `colored::*` import in `main.rs`
- [ ] Run `cargo clippy` and address all warnings
- [ ] Run `cargo fmt` on all files

### Phase 1.2: Enhanced Reporting 🟡 MEDIUM PRIORITY

#### 1.2.1 JSON Output Enhancement
**File**: `crates/deja_cli/src/commands/check.rs:145-149`
**Current**: Basic `serde_json::to_string_pretty()`

**Tasks**:
- [ ] Create a structured JSON schema for output
- [ ] Add metadata: version, timestamp, config used
- [ ] Include file statistics in JSON output
- [ ] Add summary section (total clones, duplication %, lines duplicated)
- [ ] Write JSON schema documentation
- [ ] Add JSON validation tests

**Output Structure**:
```json
{
  "version": "0.1.0",
  "timestamp": "2025-11-06T10:30:00Z",
  "config": {
    "mode": "fast",
    "min_tokens": 50,
    "min_lines": 5
  },
  "summary": {
    "files_analyzed": 10,
    "total_clones": 15,
    "clone_groups": 3,
    "duplicated_lines": 120,
    "duplication_percentage": 12.5
  },
  "clone_groups": [...]
}
```

#### 1.2.2 SARIF Output Implementation 🟡 MEDIUM
**File**: `crates/deja_cli/src/commands/check.rs:150-154`
**Current**: Placeholder "not yet implemented"

**Tasks**:
- [ ] Research SARIF 2.1.0 specification
- [ ] Create SARIF output formatter module
- [ ] Map CloneGroup to SARIF result objects
- [ ] Map Clone instances to SARIF locations
- [ ] Add tool metadata (name, version, semantic version)
- [ ] Test with SARIF validators and viewers
- [ ] Document SARIF integration in README

**Benefits**: IDE integration (VS Code, IntelliJ), CI/CD tool support

#### 1.2.3 Report Customization Options
**Tasks**:
- [ ] Add `--report-style` flag (minimal, standard, detailed)
- [ ] Add `--show-code` flag to control code snippet display
- [ ] Add `--summary-only` flag
- [ ] Add `--max-groups` to limit output size
- [ ] Add exit codes (0=no clones, 1=clones found, 2=error)
- [ ] Document all output options in CLI help

#### 1.2.4 Summary Statistics
**File**: `crates/deja_core/src/lib.rs` (DetectionResult)

**Tasks**:
- [ ] Add `total_lines` to DetectionResult
- [ ] Add `duplicated_lines` calculation
- [ ] Add `duplication_percentage` calculation
- [ ] Display metrics in CLI output
- [ ] Include in JSON/SARIF output

### Phase 1.3: Documentation & Polish 🟢 NICE-TO-HAVE

**Tasks**:
- [ ] Update README with real examples and screenshots
- [ ] Document configuration options and presets
- [ ] Add CLI usage guide with examples
- [ ] Document output formats (text, JSON, SARIF)
- [ ] Add troubleshooting guide
- [ ] Create CONTRIBUTING.md with development setup
- [ ] Add benchmark results vs other tools (aspirational)

### Phase 1.4: Testing & Quality 🟡 MEDIUM

**Tasks**:
- [ ] Add edge case tests for tokenizer
- [ ] Add JSON output format tests
- [ ] Add SARIF output format tests
- [ ] Test with real-world codebases
- [ ] Performance testing with large files (>1000 lines)
- [ ] Memory usage profiling
- [ ] Add CI/CD pipeline (GitHub Actions)

### Phase 1 Definition of Done ✅

- [ ] Token detector accurately finds Type-1 and Type-2 clones
- [ ] Line numbers are correct in all outputs
- [ ] Text, JSON, and SARIF output formats all work
- [ ] CLI has proper exit codes and error handling
- [ ] Documentation is complete and accurate
- [ ] All tests pass (unit + integration)
- [ ] Can analyze real Python projects successfully

---

## Phase 2: AST-Based Detection (BALANCED MODE)

**Goal**: Add AST-based detection for Type-3 clones (with modifications)
**Status**: Not Started

### Phase 2.1: AST Infrastructure

**Tasks**:
- [ ] Design language-agnostic AST representation
- [ ] Integrate `ruff_python_ast` from GitHub (unpublished crate)
  - Add as git dependency in Cargo.toml: `ruff_python_ast = { git = "https://github.com/astral-sh/ruff", tag = "vX.X.X" }`
  - Pin to specific Ruff release tag for stability
  - Path: `https://github.com/astral-sh/ruff/tree/main/crates/ruff_python_ast`
- [ ] Create AST normalization algorithms
- [ ] Build AST similarity metrics (tree edit distance)
- [ ] Implement AST-based detector using CloneDetector trait

**Key Challenges**:
- Handling different AST structures across languages
- Efficient tree comparison algorithms
- Balancing accuracy vs performance
- Managing unpublished crate dependency from Ruff monorepo

**Architecture Decision**:
Use Ruff's `ruff_python_ast` crate instead of RustPython for Python AST parsing. This provides:
- Production-grade Python AST parser maintained by Astral (Ruff team)
- Battle-tested with millions of Python files
- Full Python 3.12+ syntax support
- Better performance and error recovery
- Alignment with Ruff's proven architecture

**Dependency Configuration**:
```toml
[dependencies]
ruff_python_ast = { git = "https://github.com/astral-sh/ruff", tag = "v0.14.0" }
```

Note: Monitor Ruff releases and update tag periodically for latest Python syntax support.

### Phase 2.2: Type-3 Clone Detection

**Tasks**:
- [ ] Implement AST subtree matching
- [ ] Add configurable similarity thresholds for modifications
- [ ] Handle statement insertion/deletion
- [ ] Handle statement reordering detection
- [ ] Tune detection sensitivity

### Phase 2.3: Configuration File Support

**Tasks**:
- [ ] Design `.deja.toml` configuration format
- [ ] Implement configuration file parsing
- [ ] Support per-project settings
- [ ] Support ignore patterns (like .gitignore)
- [ ] Document configuration options

**Example `.deja.toml`**:
```toml
[detection]
mode = "balanced"
min_lines = 5
min_tokens = 50
threshold = 0.85
exclude_tests = true

[ignore]
patterns = [
    "**/node_modules/**",
    "**/vendor/**",
    "**/__pycache__/**"
]

[languages.python]
enabled = true
```

### Phase 2.4: Multi-Language Support Planning

**Tasks**:
- [ ] Design language plugin architecture
- [ ] Document how to add new languages
- [ ] Plan JavaScript/TypeScript support
- [ ] Plan Rust support
- [ ] Plan Java support

### Phase 2 Deliverables

- [ ] AST-based detector working for Python
- [ ] Detects Type-1, Type-2, and Type-3 clones
- [ ] Performance comparable to token-based mode
- [ ] Documentation for AST mode
- [ ] Benchmarks showing accuracy improvements

---

## Phase 3: Advanced Features & Multi-Language

**Goal**: Production-ready tool with multiple language support
**Status**: Planning

### Phase 3.1: Tree-sitter Integration

**Why Tree-sitter?**
- Unified parser for multiple languages
- Fast incremental parsing
- Error-tolerant parsing
- Active ecosystem

**Tasks**:
- [ ] Research tree-sitter grammar system
- [ ] Create adapter layer for tree-sitter ASTs
- [ ] Integrate tree-sitter-python
- [ ] Add JavaScript/TypeScript support
- [ ] Add Rust support
- [ ] Add Java support
- [ ] Add Go support

### Phase 3.2: JavaScript/TypeScript Support

**Tasks**:
- [ ] Add tree-sitter-javascript grammar
- [ ] Add tree-sitter-typescript grammar
- [ ] Implement JS/TS tokenizer
- [ ] Handle JSX/TSX syntax
- [ ] Test with popular JS/TS projects (React, Node, etc.)
- [ ] Document JS/TS specific configuration

### Phase 3.3: Rust Support

**Tasks**:
- [ ] Add tree-sitter-rust grammar
- [ ] Implement Rust tokenizer
- [ ] Handle Rust macros appropriately
- [ ] Test with popular Rust projects
- [ ] Document Rust specific configuration

### Phase 3.4: Java Support

**Tasks**:
- [ ] Add tree-sitter-java grammar
- [ ] Implement Java tokenizer
- [ ] Handle annotations and generics
- [ ] Test with enterprise Java codebases
- [ ] Document Java specific configuration

### Phase 3.5: Go Support

**Tasks**:
- [ ] Add tree-sitter-go grammar
- [ ] Implement Go tokenizer
- [ ] Handle goroutines and channels in analysis
- [ ] Test with popular Go projects
- [ ] Document Go specific configuration

### Phase 3 Deliverables

- [ ] Support for 5+ programming languages
- [ ] Unified detection across all languages
- [ ] Language-specific documentation
- [ ] Multi-language project support
- [ ] Performance benchmarks per language

---

## Future Enhancements (Post Phase 3)

### Graph-Based Detection (Type-4 Clones)

**Research Required**:
- Program Dependence Graphs (PDG)
- Control Flow Graphs (CFG)
- Graph isomorphism algorithms
- Semantic similarity metrics

**Tasks**:
- [ ] Build PDG from AST
- [ ] Implement subgraph matching
- [ ] Add semantic clone detection
- [ ] Balance precision vs recall
- [ ] Extensive testing

### IDE Integration

- [ ] VS Code extension
  - [ ] Real-time duplicate detection
  - [ ] Quick fixes and refactoring suggestions
  - [ ] Settings integration
- [ ] IntelliJ plugin
  - [ ] IntelliJ platform integration
  - [ ] Android Studio support
- [ ] Language Server Protocol (LSP)
  - [ ] Implement LSP server
  - [ ] Support for any LSP-compatible editor

### CI/CD Integration

- [ ] GitHub Actions workflow template
- [ ] GitLab CI integration
- [ ] Pre-commit hooks
- [ ] Quality gate integration
- [ ] Trend tracking over time

### Advanced Features

- [ ] Clone refactoring suggestions
  - Extract method refactoring
  - Create shared utility functions
  - Parameter object patterns
- [ ] Historical clone tracking
  - Git integration
  - Track clone introduction
  - Clone evolution analysis
- [ ] Clone visualization
  - Interactive web dashboard
  - Clone relationship graphs
  - Hotspot heatmaps
- [ ] Machine learning-based detection
  - Train on labeled clone datasets
  - Improve Type-4 detection
  - Reduce false positives
- [ ] Cross-project clone detection
  - Detect duplicates across repositories
  - Open source license compliance
  - Shared code identification

### Performance & Scalability

- [ ] Incremental analysis
  - Only analyze changed files
  - Cache detection results
  - Git-aware diffing
- [ ] Distributed analysis
  - Split large codebases
  - Parallel processing
  - Result aggregation
- [ ] Optimization
  - Memory usage profiling
  - Algorithmic improvements
  - Benchmark suite

---

## Technical Debt

### High Priority
- [ ] Fix Python tokenizer line number calculation (Phase 1.1.1)
- [ ] Fix Python bindings build (PyO3 linking issues)
- [ ] Improve error messages and error handling

### Medium Priority
- [ ] Add logging framework (tracing is initialized but underutilized)
- [ ] Add configuration file support (.deja.toml)
- [ ] Improve test fixture quality (they should actually have detectable clones)
- [ ] Add proper benchmarking infrastructure

### Low Priority
- [ ] Address all clippy warnings
- [ ] Improve code comments and inline documentation
- [ ] Reduce code duplication in test code
- [ ] Add property-based testing (proptest)

---

## Success Metrics

### Phase 1 (Token-Based Detection)
- [ ] Detects clones in 100+ line Python files in <1 second
- [ ] <5% false positive rate on curated test suite
- [ ] >90% true positive rate on known clones
- [ ] Zero crashes on valid Python code
- [ ] All three output formats working (text, JSON, SARIF)

### Phase 2 (AST-Based Detection)
- [ ] Detects Type-3 clones with 80%+ accuracy
- [ ] Performance within 3x of token-based mode
- [ ] Handles files up to 5000 lines efficiently
- [ ] False positive rate <10%

### Phase 3 (Multi-Language)
- [ ] Support 5+ languages
- [ ] Consistent detection quality across languages
- [ ] Performance <5 min for 100k LOC project
- [ ] Active community adoption (100+ GitHub stars)

### Long-Term
- [ ] Competitive with commercial tools (SonarQube, PMD)
- [ ] Used in production CI/CD pipelines
- [ ] IDE extensions with 1000+ installs
- [ ] Published academic paper or technical report

---

## Resources & References

### Academic Papers
- "Comparison and Evaluation of Clone Detection Techniques" (Roy et al., 2007)
- "CCFinder: A Multi-linguistic Token-based Code Clone Detection System" (Kamiya et al., 2002)
- "Deckard: Scalable and Accurate Tree-based Detection of Code Clones" (Jiang et al., 2007)
- "Detecting Code Clones with Graph Neural Networks" (Wang et al., 2020)

### Tools for Comparison
- **PMD CPD** - Token-based, multi-language
- **SonarQube** - Commercial, comprehensive
- **Simian** - Simple, fast
- **CloneDR** - Commercial, AST-based
- **SourcererCC** - Large-scale token-based
- **NiCad** - Text-based with AST filtering

### Technical References
- SARIF 2.1.0 Specification: https://docs.oasis-open.org/sarif/sarif/v2.1.0/
- Tree-sitter: https://tree-sitter.github.io/tree-sitter/
- RustPython Parser: https://github.com/RustPython/Parser

### Inspiration & Architecture
- **Ruff** - Rust Python linter, architecture model
- **ripgrep** - Performance and CLI design
- **ast-grep** - AST pattern matching
- **cargo-deny** - Rust workspace tooling

---

## Development Guidelines

### Code Standards
- Follow Rust API Guidelines
- Use clippy and rustfmt
- Write comprehensive tests (unit + integration)
- Document public APIs with examples
- Keep functions focused and small (<100 lines)

### Testing Philosophy
- Test-driven development for core algorithms
- Integration tests for end-to-end workflows
- Property-based testing for invariants
- Benchmark critical paths

### Performance Targets
- Token detection: O(n) where n = tokens
- AST detection: O(n²) worst case, O(n log n) typical
- Memory usage: <500MB for 100k LOC project
- Startup time: <100ms

### Documentation Standards
- User documentation in `docs/`
- API documentation via rustdoc
- Architecture decision records (ADRs)
- Changelog following Keep a Changelog

---

## Community & Contribution

### Contribution Priorities
1. Bug fixes and error handling
2. Documentation improvements
3. Test coverage expansion
4. New language support
5. Performance optimizations
6. New features (after discussion)

### Getting Started (for contributors)
1. Read CONTRIBUTING.md
2. Check issues labeled "good first issue"
3. Set up development environment
4. Run test suite to verify setup
5. Pick an issue and submit PR

### Communication
- GitHub Issues: Bug reports, feature requests
- GitHub Discussions: Questions, ideas
- Pull Requests: Code contributions
- Changelog: Track all changes

---

## Notes

- **Focus on correctness over performance initially** - get algorithms right first
- **Keep architecture clean** for multi-language support in later phases
- **Build comprehensive test suite** from the start - it pays off
- **Document algorithms and decisions** for long-term maintainability
- **Engage with community early** for feedback and adoption
- **Regular benchmarking** against other tools to track progress

---

## Version History

- **v0.1.0** (Current): Phase 1 in progress - token-based detection
- **v0.2.0** (Planned): Phase 1 complete
- **v0.3.0** (Planned): Phase 2 complete - AST detection
- **v0.4.0** (Planned): Phase 3 complete - multi-language
- **v1.0.0** (Goal): Production-ready with IDE integrations

---

**Last Updated**: 2025-11-06
**Maintained By**: Deja-vu Development Team
**Status**: Active Development
