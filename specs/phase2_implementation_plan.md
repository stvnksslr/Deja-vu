# Phase 2 Implementation Plan: AST-Based Detection (BALANCED MODE)

**Created**: 2025-11-06
**Status**: Planning
**Estimated Timeline**: 3-4 weeks
**Dependencies**: Phase 1 completion (85% complete)

---

## Executive Summary

Phase 2 will add AST-based clone detection to enable detection of Type-3 clones (code with modifications like statement insertions/deletions). This complements the existing token-based Type-1/Type-2 detection with more sophisticated structural analysis.

**Key Deliverables**:
1. AST-based detector for Python using Ruff's AST
2. AST normalization and tree comparison algorithms
3. Configuration file support (.deja.toml)
4. Enhanced detection accuracy for Type-3 clones
5. Foundation for multi-language support

---

## Phase 2.1: AST Infrastructure (Week 1-2)

### 2.1.1: Integrate Ruff's Python AST Parser

**Priority**: 🔴 CRITICAL - Foundation for all AST work

**Tasks**:

1. **Add Ruff AST Dependency**
   - File: `crates/deja_python/Cargo.toml`
   - Add git dependency:
     ```toml
     [dependencies]
     ruff_python_ast = { git = "https://github.com/astral-sh/ruff", tag = "v0.7.4" }
     ruff_python_parser = { git = "https://github.com/astral-sh/ruff", tag = "v0.7.4" }
     ruff_text_size = { git = "https://github.com/astral-sh/ruff", tag = "v0.7.4" }
     ```
   - Research latest stable Ruff release tag
   - Pin to specific version for reproducibility

2. **Create AST Parser Module**
   - New file: `crates/deja_python/src/parser.rs`
   - Implement `parse_python_ast()` function
   - Handle parsing errors gracefully
   - Convert Ruff AST to our internal representation

3. **Test AST Parsing**
   - New file: `crates/deja_python/tests/ast_parsing_test.rs`
   - Test with various Python syntax:
     - Functions, classes, methods
     - Control flow (if/for/while)
     - Exception handling
     - Comprehensions
     - Decorators
   - Verify line numbers and positions are preserved

**Acceptance Criteria**:
- [ ] Ruff AST dependency compiles successfully
- [ ] Can parse Python files into AST
- [ ] Preserves source location information
- [ ] Handles syntax errors without crashing
- [ ] Tests pass for all Python syntax variants

---

### 2.1.2: Design Language-Agnostic AST Representation

**Priority**: 🟡 HIGH - Enables multi-language support

**Tasks**:

1. **Create Generic AST Module**
   - New file: `crates/deja_ast/src/generic_ast.rs`
   - Define core AST node types:
     ```rust
     pub enum AstNode {
         Function { name: String, params: Vec<String>, body: Vec<AstNode> },
         Class { name: String, methods: Vec<AstNode> },
         IfStatement { condition: Box<AstNode>, then_branch: Vec<AstNode>, else_branch: Vec<AstNode> },
         Loop { kind: LoopKind, body: Vec<AstNode> },
         Assignment { target: String, value: Box<AstNode> },
         Call { func: String, args: Vec<AstNode> },
         Literal { value: LiteralValue },
         // ... more node types
     }
     ```
   - Include metadata: line/col, span, source text

2. **Create Python → Generic AST Converter**
   - New file: `crates/deja_python/src/ast_converter.rs`
   - Implement visitor pattern for Ruff AST
   - Map Python AST nodes to generic nodes
   - Preserve structural information
   - Handle Python-specific constructs (comprehensions, decorators)

3. **Add AST Normalization**
   - Normalize identifiers (like token-based `$ID`)
   - Normalize literals (`$LIT`)
   - Remove comments and docstrings
   - Preserve structure but abstract values

**Acceptance Criteria**:
- [ ] Generic AST can represent common programming constructs
- [ ] Python AST converts to generic AST
- [ ] Normalization preserves code structure
- [ ] Round-trip testing (Python → Generic → comparison works)
- [ ] Documentation explains AST design choices

---

### 2.1.3: Build AST Similarity Metrics

**Priority**: 🟡 HIGH - Core algorithm for Type-3 detection

**Tasks**:

1. **Implement Tree Edit Distance**
   - New file: `crates/deja_core/src/tree_edit_distance.rs`
   - Implement Zhang-Shasha algorithm or similar
   - Operations: insert, delete, update node
   - Calculate minimum edit distance between trees
   - Reference: "Simple Fast Algorithms for the Editing Distance Between Trees" (Zhang & Shasha, 1989)

2. **Create AST Similarity Scorer**
   - New file: `crates/deja_core/src/ast_similarity.rs`
   - Normalize edit distance to 0.0-1.0 similarity score
   - Weight different node types differently:
     - Structure changes (control flow): high weight
     - Value changes (literals): low weight
     - Identifier changes: medium weight
   - Configurable similarity thresholds

3. **AST Hashing for Candidate Selection**
   - Hash AST subtrees for quick comparison
   - Similar to token rolling hash but for trees
   - Use structural fingerprints (node types + depth)
   - Fast candidate filtering before expensive comparison

**Acceptance Criteria**:
- [ ] Tree edit distance implementation correct
- [ ] Similarity score makes intuitive sense (0.0 = different, 1.0 = identical)
- [ ] AST hashing finds similar subtrees quickly
- [ ] Performance acceptable for medium trees (50-100 nodes)
- [ ] Unit tests verify algorithm correctness

---

### 2.1.4: Implement AST-Based Detector

**Priority**: 🟡 HIGH - Core functionality

**Tasks**:

1. **Create AstBasedDetector**
   - New file: `crates/deja_core/src/ast_detector.rs`
   - Implement `CloneDetector` trait
   - Similar structure to `TokenBasedDetector`:
     ```rust
     pub struct AstBasedDetector {
         parsers: HashMap<String, Box<dyn LanguageParser>>,
     }

     impl CloneDetector for AstBasedDetector {
         fn detect(&self, files: &[SourceFile], config: &DetectionConfig) -> Result<Vec<CloneGroup>> {
             // 1. Parse files to ASTs
             // 2. Extract all subtrees above min size
             // 3. Hash subtrees for candidate pairs
             // 4. Compare candidates using tree edit distance
             // 5. Group similar subtrees
             // 6. Filter by thresholds
         }
     }
     ```

2. **Define LanguageParser Trait**
   - New file: `crates/deja_ast/src/parser.rs`
   - Generic interface for language parsers:
     ```rust
     pub trait LanguageParser: Send + Sync {
         fn parse(&self, source: &str) -> Result<AstNode, ParsingError>;
         fn language(&self) -> &str;
     }
     ```
   - Implement for Python first

3. **AST Clone Extraction**
   - Extract all subtrees of configurable min size
   - Minimum nodes per subtree (e.g., 10 nodes)
   - Skip trivial subtrees (single assignments, returns)
   - Preserve source location for reporting

4. **Integration with CLI**
   - Update `crates/deja_cli/src/commands/check.rs`
   - Use AST detector for "balanced" mode
   - Keep token detector for "fast" mode
   - Add mode selection logic

**Acceptance Criteria**:
- [ ] AST detector implements CloneDetector trait
- [ ] Detects Type-3 clones (with statement insertions/deletions)
- [ ] Performance within 3x of token-based mode
- [ ] CLI can run in balanced mode
- [ ] Integration tests pass

---

## Phase 2.2: Type-3 Clone Detection Tuning (Week 2)

### 2.2.1: Subtree Matching Algorithms

**Tasks**:

1. **Implement Exact Subtree Matching**
   - Find identical AST subtrees (Type-1 at AST level)
   - Use hashing for O(n) performance
   - Baseline for comparison

2. **Implement Fuzzy Subtree Matching**
   - Allow small differences in subtrees
   - Configurable similarity threshold (0.8 default)
   - Handle:
     - Different variable names (Type-2)
     - Different literal values (Type-2)
     - Extra/missing statements (Type-3)
     - Reordered statements (Type-3)

3. **Subtree Size Heuristics**
   - Min nodes: 10 (configurable)
   - Max depth: unlimited (but track performance)
   - Skip boilerplate patterns at AST level:
     - Simple getters/setters
     - Empty constructors
     - Pass-through methods

**Acceptance Criteria**:
- [ ] Exact matching works correctly
- [ ] Fuzzy matching finds reasonable clones
- [ ] Size heuristics filter noise
- [ ] Performance acceptable on real codebases

---

### 2.2.2: Handle Modifications

**Tasks**:

1. **Statement Insertion/Deletion Detection**
   - Compare subtrees with some statements missing
   - Example: same logic but with/without logging
   - Weight insertions/deletions appropriately
   - Don't penalize defensive programming (extra checks)

2. **Statement Reordering Detection**
   - Detect when statement order differs but logic same
   - Example: variable declarations in different order
   - Use edit distance permutation awareness
   - Careful with control flow changes

3. **Parameter and Argument Matching**
   - Functions with different parameter names → Type-2
   - Functions with different parameter counts → evaluate similarity
   - Default arguments handling

**Acceptance Criteria**:
- [ ] Detects clones with extra/missing logging statements
- [ ] Detects clones with reordered declarations
- [ ] Doesn't flag significantly different logic
- [ ] Test cases demonstrate each capability

---

### 2.2.3: Threshold Calibration

**Tasks**:

1. **Create Calibration Test Suite**
   - New file: `tests/fixtures/ast_clones/`
   - Known Type-3 clone pairs
   - Known non-clones (similar but different)
   - Edge cases

2. **Tune Similarity Thresholds**
   - Test different threshold values (0.7, 0.75, 0.8, 0.85)
   - Measure precision and recall
   - Find sweet spot balancing false positives/negatives
   - Document recommended thresholds

3. **Add Confidence Scores**
   - Report confidence for each clone group
   - High confidence: >0.9 similarity
   - Medium: 0.8-0.9
   - Low: 0.7-0.8
   - Allow filtering by confidence

**Acceptance Criteria**:
- [ ] Thresholds validated on test suite
- [ ] <10% false positive rate
- [ ] >80% true positive rate on Type-3 clones
- [ ] Confidence scores correlate with human judgment

---

## Phase 2.3: Configuration File Support (Week 3)

### 2.3.1: Design Configuration Format

**Tasks**:

1. **Create .deja.toml Specification**
   - New file: `specs/config_format.md`
   - Document all configuration options
   - Example configurations for common scenarios
   - Validation rules

2. **Example Configuration**:
   ```toml
   # .deja.toml - Deja-vu Configuration

   [detection]
   # Detection mode: "fast" (token), "balanced" (AST), "precise" (graph - future)
   mode = "balanced"

   # Minimum lines for a clone
   min_lines = 5

   # Minimum tokens for token-based detection
   min_tokens = 20

   # Minimum AST nodes for AST-based detection
   min_nodes = 10

   # Similarity threshold (0.0 - 1.0)
   threshold = 0.85

   # Ignore comments and whitespace
   ignore_comments = true
   ignore_whitespace = true

   # Skip test files
   exclude_tests = false

   [output]
   # Output format: "text", "json", "sarif"
   format = "text"

   # Verbosity level: "quiet", "normal", "verbose"
   verbosity = "normal"

   # Show code snippets in output
   show_code = true

   # Maximum clone groups to show
   max_groups = 100

   [ignore]
   # Patterns to ignore (glob syntax)
   patterns = [
       "**/node_modules/**",
       "**/vendor/**",
       "**/__pycache__/**",
       "**/build/**",
       "**/dist/**",
       "**/.venv/**",
   ]

   # Specific files to ignore
   files = [
       "migrations/*.py",
   ]

   [languages.python]
   enabled = true
   # Python-specific settings
   ignore_docstrings = true

   [boilerplate]
   # Filter common boilerplate patterns
   filter_imports = true
   filter_test_setup = true
   filter_simple_declarations = true

   # Custom boilerplate patterns (regex)
   custom_patterns = [
       "logger\\..*",  # Logging statements
   ]
   ```

**Acceptance Criteria**:
- [ ] Configuration format is well-documented
- [ ] Covers all major use cases
- [ ] Backwards compatible (reasonable defaults)
- [ ] Validated against schema

---

### 2.3.2: Implement Configuration Parser

**Tasks**:

1. **Add TOML Dependency**
   - File: `Cargo.toml`
   - Add `toml = "0.8"` and `serde = { version = "1.0", features = ["derive"] }`

2. **Create Configuration Module**
   - New file: `crates/deja_core/src/config.rs`
   - Define structs matching TOML format:
     ```rust
     #[derive(Debug, Deserialize)]
     pub struct DejaConfig {
         pub detection: DetectionSettings,
         pub output: OutputSettings,
         pub ignore: IgnoreSettings,
         pub languages: HashMap<String, LanguageSettings>,
         pub boilerplate: BoilerplateSettings,
     }
     ```
   - Implement validation logic
   - Provide sensible defaults

3. **Configuration Loading**
   - New file: `crates/deja_core/src/config_loader.rs`
   - Search for `.deja.toml` in:
     1. Current directory
     2. Parent directories (walk up to git root)
     3. User home directory (`~/.deja.toml`)
   - Merge configurations (project overrides user)
   - CLI flags override config file

4. **CLI Integration**
   - Update `crates/deja_cli/src/main.rs`
   - Add `--config` flag to specify custom config file
   - Load config before processing
   - Allow CLI flags to override config values

**Acceptance Criteria**:
- [ ] Can parse valid .deja.toml files
- [ ] Validation catches invalid configurations
- [ ] Config search finds files correctly
- [ ] CLI flags override config file
- [ ] Tests verify all configuration scenarios

---

### 2.3.3: Implement Ignore Patterns

**Tasks**:

1. **Add Glob Pattern Matching**
   - Add dependency: `globset = "0.4"`
   - File: `crates/deja_core/src/ignore.rs`
   - Support glob patterns like gitignore
   - Support negation patterns (!pattern)

2. **Integrate with File Collection**
   - Update `crates/deja_core/src/utils.rs::collect_files()`
   - Apply ignore patterns during collection
   - Respect .gitignore if present
   - Add `--no-ignore` flag to disable

3. **Pattern Validation**
   - Validate patterns at config load time
   - Warn about overly broad patterns
   - Log which files are ignored (verbose mode)

**Acceptance Criteria**:
- [ ] Glob patterns work like gitignore
- [ ] Can ignore specific directories/files
- [ ] Negation patterns work
- [ ] Integration tests verify ignoring works

---

## Phase 2.4: Multi-Language Support Planning (Week 3-4)

### 2.4.1: Design Language Plugin Architecture

**Tasks**:

1. **Define Plugin Interface**
   - File: `crates/deja_ast/src/plugin.rs`
   - Traits for language support:
     ```rust
     pub trait LanguagePlugin: Send + Sync {
         fn name(&self) -> &str;
         fn file_extensions(&self) -> &[&str];
         fn tokenizer(&self) -> Box<dyn LanguageTokenizer>;
         fn parser(&self) -> Option<Box<dyn LanguageParser>>;
         fn normalizer(&self) -> Option<Box<dyn AstNormalizer>>;
     }
     ```

2. **Plugin Registry**
   - New file: `crates/deja_core/src/plugin_registry.rs`
   - Register and discover plugins
   - Load plugins dynamically (future: support external plugins)
   - Route files to correct plugin by extension

3. **Python Plugin Implementation**
   - New file: `crates/deja_python/src/plugin.rs`
   - Implement `LanguagePlugin` for Python
   - Use existing tokenizer and new AST parser
   - Reference implementation for other languages

**Acceptance Criteria**:
- [ ] Plugin architecture is extensible
- [ ] Python plugin works end-to-end
- [ ] Documentation explains how to add languages
- [ ] Clear separation of concerns

---

### 2.4.2: Document Language Addition Process

**Tasks**:

1. **Create Language Addition Guide**
   - New file: `docs/adding_languages.md`
   - Step-by-step guide:
     1. Choose parser (tree-sitter recommended)
     2. Implement LanguageTokenizer
     3. Implement LanguageParser (for AST mode)
     4. Map to generic AST
     5. Add tests
     6. Register plugin
   - Example: Adding JavaScript support

2. **Create Template Files**
   - Template: `crates/deja_LANGUAGE/`
   - Boilerplate for new language crates
   - Copy-paste starting point

**Acceptance Criteria**:
- [ ] Guide is clear and comprehensive
- [ ] Templates reduce boilerplate
- [ ] Documented process is validated

---

### 2.4.3: Plan JavaScript/TypeScript Support

**Research and Planning Tasks**:

1. **Research Tree-sitter Integration**
   - Evaluate tree-sitter-rust bindings
   - Test tree-sitter-javascript grammar
   - Test tree-sitter-typescript grammar
   - Prototype AST extraction

2. **Design JS/TS AST Mapping**
   - Map JS/TS constructs to generic AST
   - Handle JSX/TSX syntax
   - Handle async/await, generators
   - Handle ES modules, CommonJS

3. **Create Implementation Plan**
   - New file: `specs/javascript_support.md`
   - Detailed tasks for JS/TS integration
   - Timeline estimate
   - Dependencies and blockers

**Deliverable**: Detailed plan for Phase 3 JS/TS support

---

## Phase 2.5: Testing & Quality (Week 4)

### 2.5.1: Comprehensive Test Suite

**Tasks**:

1. **AST Unit Tests**
   - Test all AST node conversions
   - Test tree edit distance algorithm
   - Test similarity scoring
   - Test normalization

2. **Integration Tests**
   - End-to-end AST detection tests
   - Test with real Python files
   - Type-3 clone detection scenarios
   - Configuration file loading tests

3. **Performance Tests**
   - Benchmark AST vs token detection
   - Profile memory usage
   - Test with large files (1000+ lines)
   - Ensure <3x slower than token mode

4. **Regression Tests**
   - Ensure Phase 1 token detection still works
   - No performance regression in fast mode
   - Backward compatibility

**Acceptance Criteria**:
- [ ] >90% code coverage for new code
- [ ] All tests pass consistently
- [ ] Performance within targets
- [ ] CI/CD pipeline green

---

### 2.5.2: Documentation

**Tasks**:

1. **Update README.md**
   - Document balanced mode
   - Show AST detection examples
   - Update installation instructions
   - Add configuration examples

2. **Create AST Detection Guide**
   - New file: `docs/ast_detection.md`
   - Explain how AST detection works
   - When to use balanced vs fast mode
   - Tuning for best results

3. **API Documentation**
   - Rustdoc comments for all public APIs
   - Examples in doc comments
   - Generate docs: `cargo doc --no-deps`

**Acceptance Criteria**:
- [ ] Documentation is complete and accurate
- [ ] Examples work
- [ ] API docs generated successfully

---

## Implementation Order & Dependencies

### Week 1: AST Foundation
1. ✅ Add Ruff AST dependency
2. ✅ Create AST parser module
3. ✅ Design generic AST
4. ✅ Python → Generic AST converter
5. ✅ AST normalization

**Milestone**: Can parse Python files to normalized AST

### Week 2: Detection Algorithm
1. ✅ Tree edit distance implementation
2. ✅ AST similarity scoring
3. ✅ AST hashing for candidates
4. ✅ AST-based detector implementation
5. ✅ CLI integration

**Milestone**: AST detector finds Type-3 clones

### Week 3: Configuration & Polish
1. ✅ Configuration format design
2. ✅ Configuration parser
3. ✅ Ignore patterns
4. ✅ Plugin architecture
5. ✅ Threshold calibration

**Milestone**: Production-ready configuration system

### Week 4: Testing & Documentation
1. ✅ Comprehensive test suite
2. ✅ Performance benchmarking
3. ✅ Documentation
4. ✅ Multi-language planning
5. ✅ Phase 2 completion review

**Milestone**: Phase 2 complete and documented

---

## Success Criteria - Phase 2 Complete

### Functional Requirements
- [ ] AST-based detector works for Python
- [ ] Detects Type-1, Type-2, and Type-3 clones
- [ ] Configuration file support (.deja.toml)
- [ ] Ignore patterns work
- [ ] Balanced mode accessible via CLI

### Quality Requirements
- [ ] Performance within 3x of token mode
- [ ] False positive rate <10%
- [ ] True positive rate >80% on Type-3 clones
- [ ] All tests pass
- [ ] Code coverage >85%

### Documentation Requirements
- [ ] README updated with AST mode
- [ ] Configuration guide complete
- [ ] Language addition guide complete
- [ ] API docs generated
- [ ] Examples work

### Architecture Requirements
- [ ] Plugin architecture enables new languages
- [ ] Clean separation: core, language-specific, CLI
- [ ] Extensible for Phase 3
- [ ] No technical debt introduced

---

## Risk Assessment

### High Risk
- **Tree edit distance performance**: May be too slow for large ASTs
  - *Mitigation*: Use hashing for candidate filtering, optimize algorithm

- **Ruff AST changes**: Unpublished crate may have breaking changes
  - *Mitigation*: Pin to specific git tag, test before updating

### Medium Risk
- **Configuration complexity**: Too many options may confuse users
  - *Mitigation*: Good defaults, clear documentation, examples

- **False positives**: AST may match too broadly
  - *Mitigation*: Careful threshold tuning, user feedback

### Low Risk
- **Multi-language planning**: Unknown complexity for other languages
  - *Mitigation*: Research phase before implementation

---

## Phase 2 Completion Checklist

### Code
- [ ] All Phase 2.1 tasks complete
- [ ] All Phase 2.2 tasks complete
- [ ] All Phase 2.3 tasks complete
- [ ] All Phase 2.4 planning tasks complete
- [ ] Code reviewed and approved
- [ ] No clippy warnings
- [ ] cargo fmt applied

### Testing
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Performance tests pass
- [ ] No regression in Phase 1 functionality

### Documentation
- [ ] README updated
- [ ] Configuration guide written
- [ ] Language addition guide written
- [ ] API docs complete
- [ ] CHANGELOG updated

### Review
- [ ] Peer code review
- [ ] Architecture review
- [ ] User acceptance testing
- [ ] Performance validated
- [ ] Ready for Phase 3

---

## Next Steps (Phase 3 Preview)

After Phase 2 completion:
1. Tree-sitter integration research
2. JavaScript/TypeScript support
3. Rust language support
4. IDE integration (VS Code extension)
5. Type-4 semantic clone detection (future)

---

**Last Updated**: 2025-11-06
**Author**: Claude Code
**Status**: Planning Complete - Ready for Implementation
