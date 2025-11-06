//! Python tokenizer for token-based clone detection

use deja_core::token::{LanguageTokenizer, Token, TokenType, TokenizationError};
use rustpython_parser::lexer::{lex, LexResult};
use rustpython_parser::Mode;

/// Python tokenizer
pub struct PythonTokenizer;

impl PythonTokenizer {
    pub fn new() -> Self {
        Self
    }

    /// Build a line index mapping byte offsets to (line, column) positions
    fn build_line_index(source: &str) -> Vec<usize> {
        let mut line_starts = vec![0];
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(idx + 1);
            }
        }
        line_starts
    }

    /// Convert byte offset to (line, column) using the line index
    fn offset_to_position(offset: usize, line_starts: &[usize]) -> (usize, usize) {
        // Binary search to find which line this offset belongs to
        let line = match line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };

        let line_start = line_starts[line];
        let column = offset.saturating_sub(line_start);

        (line + 1, column) // Convert to 1-based line numbers
    }

    /// Tokenize Python source code
    fn tokenize_python(&self, source: &str) -> Result<Vec<Token>, TokenizationError> {
        let tokens: Vec<LexResult> = lex(source, Mode::Module).collect();
        let line_starts = Self::build_line_index(source);

        let mut result = Vec::new();
        let mut docstring_detector = DocstringDetector::new();

        for token_result in tokens {
            match token_result {
                Ok((tok, range)) => {
                    let start_offset = range.start().to_usize();
                    let end_offset = range.end().to_usize();

                    let (start_line, start_col) =
                        Self::offset_to_position(start_offset, &line_starts);

                    // Check if this token is a docstring
                    let is_docstring = docstring_detector.is_docstring(&tok);

                    // Get the token type, but override if it's a docstring
                    let token_type = if is_docstring {
                        TokenType::Comment
                    } else {
                        self.map_token_type(&tok)
                    };

                    let value = self.token_to_string(&tok);

                    result.push(Token::new(
                        token_type,
                        value,
                        start_offset,
                        end_offset,
                        start_line,
                        start_col,
                    ));
                }
                Err(e) => {
                    return Err(TokenizationError::LexError(format!("{:?}", e)));
                }
            }
        }

        Ok(result)
    }

    fn token_to_string(&self, tok: &rustpython_parser::Tok) -> String {
        use rustpython_parser::Tok;

        match tok {
            Tok::Name { name } => name.to_string(),
            Tok::Int { value } => value.to_string(),
            Tok::Float { value } => value.to_string(),
            Tok::Complex { real, imag } => format!("{}+{}j", real, imag),
            Tok::String { value, .. } => value.to_string(),
            Tok::Newline => "\n".to_string(),
            Tok::Indent => "    ".to_string(),
            Tok::Dedent => "".to_string(),
            _ => format!("{:?}", tok),
        }
    }

    fn map_token_type(&self, tok: &rustpython_parser::Tok) -> TokenType {
        use rustpython_parser::Tok;

        match tok {
            // Keywords
            Tok::And
            | Tok::Or
            | Tok::Not
            | Tok::Is
            | Tok::In
            | Tok::If
            | Tok::Else
            | Tok::Elif
            | Tok::While
            | Tok::For
            | Tok::Def
            | Tok::Class
            | Tok::Return
            | Tok::Yield
            | Tok::Import
            | Tok::From
            | Tok::As
            | Tok::Try
            | Tok::Except
            | Tok::Finally
            | Tok::With
            | Tok::Lambda
            | Tok::Pass
            | Tok::Break
            | Tok::Continue
            | Tok::Global
            | Tok::Nonlocal
            | Tok::Assert
            | Tok::Del
            | Tok::Raise
            | Tok::Async
            | Tok::Await
            | Tok::Match
            | Tok::Case => TokenType::Keyword,

            // Identifiers
            Tok::Name { .. } => TokenType::Identifier,

            // Literals
            Tok::Int { .. }
            | Tok::Float { .. }
            | Tok::Complex { .. }
            | Tok::String { .. }
            | Tok::True
            | Tok::False
            | Tok::None => TokenType::Literal,

            // Operators
            Tok::Plus
            | Tok::Minus
            | Tok::Star
            | Tok::Slash
            | Tok::DoubleSlash
            | Tok::Percent
            | Tok::DoubleStar
            | Tok::Vbar
            | Tok::Amper
            | Tok::CircumFlex
            | Tok::LeftShift
            | Tok::RightShift
            | Tok::Less
            | Tok::Greater
            | Tok::LessEqual
            | Tok::GreaterEqual
            | Tok::EqEqual
            | Tok::NotEqual
            | Tok::Equal
            | Tok::PlusEqual
            | Tok::MinusEqual
            | Tok::StarEqual
            | Tok::SlashEqual
            | Tok::DoubleSlashEqual
            | Tok::PercentEqual
            | Tok::AmperEqual
            | Tok::VbarEqual
            | Tok::CircumflexEqual
            | Tok::LeftShiftEqual
            | Tok::RightShiftEqual
            | Tok::DoubleStarEqual
            | Tok::Rarrow
            | Tok::ColonEqual
            | Tok::Dot
            | Tok::Ellipsis
            | Tok::At
            | Tok::AtEqual
            | Tok::Tilde => TokenType::Operator,

            // Delimiters
            Tok::Lpar
            | Tok::Rpar
            | Tok::Lsqb
            | Tok::Rsqb
            | Tok::Lbrace
            | Tok::Rbrace
            | Tok::Comma
            | Tok::Colon
            | Tok::Semi => TokenType::Delimiter,

            // Whitespace
            Tok::Newline | Tok::Indent | Tok::Dedent => TokenType::Whitespace,

            // Unknown
            _ => TokenType::Unknown,
        }
    }
}

/// State machine for detecting Python docstrings
#[derive(Debug, Clone, Copy, PartialEq)]
enum DocstringState {
    Module,              // At module level, first string is module docstring
    AfterDef,            // Just saw 'def' keyword
    AfterDefColon,       // Just saw ':' after def
    AfterDefIndent,      // Just saw indent after def:, next string is function docstring
    AfterClass,          // Just saw 'class' keyword
    AfterClassColon,     // Just saw ':' after class
    AfterClassIndent,    // Just saw indent after class:, next string is class docstring
    InCode,              // In regular code, strings are not docstrings
}

/// Detector for identifying Python docstrings in token stream
struct DocstringDetector {
    state: DocstringState,
    seen_code: bool,  // Track if we've seen any code at module level
}

impl DocstringDetector {
    fn new() -> Self {
        Self {
            state: DocstringState::Module,
            seen_code: false,
        }
    }

    /// Check if the current token is a docstring and update state
    /// Returns true if the token is in a docstring position
    fn is_docstring(&mut self, tok: &rustpython_parser::Tok) -> bool {
        use rustpython_parser::Tok;

        // Check if current token is a string in docstring position
        let is_string = matches!(tok, Tok::String { .. });

        let is_docstring = match self.state {
            DocstringState::Module if is_string && !self.seen_code => true,
            DocstringState::AfterDefIndent if is_string => true,
            DocstringState::AfterClassIndent if is_string => true,
            _ => false,
        };

        // Update state based on token
        self.update_state(tok);

        is_docstring
    }

    /// Update the state machine based on the current token
    fn update_state(&mut self, tok: &rustpython_parser::Tok) {
        use rustpython_parser::Tok;

        match (self.state, tok) {
            // Function definitions (must come before Module catch-all)
            (_, Tok::Def) => {
                self.state = DocstringState::AfterDef;
            }

            // Class definitions (must come before Module catch-all)
            (_, Tok::Class) => {
                self.state = DocstringState::AfterClass;
            }

            // Module level: track first string
            (DocstringState::Module, Tok::String { .. }) if !self.seen_code => {
                self.state = DocstringState::InCode;
                self.seen_code = true;
            }
            (DocstringState::Module, _) if !matches!(tok, Tok::Newline | Tok::Indent | Tok::Dedent) => {
                self.seen_code = true;
            }
            (DocstringState::AfterDef, Tok::Colon) => {
                self.state = DocstringState::AfterDefColon;
            }
            (DocstringState::AfterDefColon, Tok::Newline) => {
                // Stay in AfterDefColon
            }
            (DocstringState::AfterDefColon, Tok::Indent) => {
                self.state = DocstringState::AfterDefIndent;
            }
            (DocstringState::AfterDefIndent, Tok::String { .. }) => {
                self.state = DocstringState::InCode;
            }
            (DocstringState::AfterDefIndent, _) if !matches!(tok, Tok::Newline | Tok::Indent) => {
                self.state = DocstringState::InCode;
            }

            (DocstringState::AfterClass, Tok::Colon) => {
                self.state = DocstringState::AfterClassColon;
            }
            (DocstringState::AfterClassColon, Tok::Newline) => {
                // Stay in AfterClassColon
            }
            (DocstringState::AfterClassColon, Tok::Indent) => {
                self.state = DocstringState::AfterClassIndent;
            }
            (DocstringState::AfterClassIndent, Tok::String { .. }) => {
                self.state = DocstringState::InCode;
            }
            (DocstringState::AfterClassIndent, _) if !matches!(tok, Tok::Newline | Tok::Indent) => {
                self.state = DocstringState::InCode;
            }

            // Stay in InCode for most other cases
            _ => {}
        }
    }
}

impl Default for PythonTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageTokenizer for PythonTokenizer {
    fn tokenize(&self, source: &str) -> Result<Vec<Token>, TokenizationError> {
        self.tokenize_python(source)
    }

    fn language(&self) -> &str {
        "python"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_python() {
        let tokenizer = PythonTokenizer::new();
        let source = "def hello():\n    pass";

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());

        // Should have 'def', 'hello', '(', ')', ':', newline, indent, 'pass', dedent
        let keyword_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Keyword)
            .count();
        assert!(keyword_count >= 2); // 'def' and 'pass'
    }

    #[test]
    fn test_tokenize_with_literals() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"x = 42"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let literal_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Literal)
            .count();
        assert!(literal_count >= 1); // At least the number 42
    }

    #[test]
    fn test_line_numbers_are_correct() {
        let tokenizer = PythonTokenizer::new();
        let source = "def hello():\n    print('world')\n    return 42";

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the 'def' token - should be on line 1
        let def_token = tokens.iter().find(|t| t.value == "Def").unwrap();
        assert_eq!(def_token.line, 1, "def should be on line 1");

        // Find the 'print' token - should be on line 2
        let print_token = tokens.iter().find(|t| t.value == "print").unwrap();
        assert_eq!(print_token.line, 2, "print should be on line 2");

        // Find the 'return' token - should be on line 3
        let return_token = tokens.iter().find(|t| t.value == "Return").unwrap();
        assert_eq!(return_token.line, 3, "return should be on line 3");

        // Find the number 42 - should also be on line 3
        let num_token = tokens.iter().find(|t| t.value == "42").unwrap();
        assert_eq!(num_token.line, 3, "42 should be on line 3");
    }

    #[test]
    fn test_column_numbers_are_correct() {
        let tokenizer = PythonTokenizer::new();
        let source = "x = 42 + y";

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // 'x' should be at column 0
        let x_token = tokens.iter().find(|t| t.value == "x").unwrap();
        assert_eq!(x_token.column, 0, "x should be at column 0");

        // '=' should be at column 2
        let eq_token = tokens
            .iter()
            .find(|t| matches!(t.token_type, TokenType::Operator) && t.start == 2)
            .unwrap();
        assert_eq!(eq_token.column, 2, "= should be at column 2");

        // '42' should be at column 4
        let num_token = tokens.iter().find(|t| t.value == "42").unwrap();
        assert_eq!(num_token.column, 4, "42 should be at column 4");

        // '+' should be at column 7
        let plus_token = tokens.iter().find(|t| t.start == 7).unwrap();
        assert_eq!(plus_token.column, 7, "+ should be at column 7");

        // 'y' should be at column 9
        let y_token = tokens.iter().find(|t| t.value == "y").unwrap();
        assert_eq!(y_token.column, 9, "y should be at column 9");
    }

    #[test]
    fn test_tokenize_simple_duplicate_file() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"def add(a, b):
    """Add two numbers"""
    result = a + b
    return result


def sum_values(x, y):
    """Sum two values (duplicate with renamed variables)"""
    result = x + y
    return result
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        println!("\n=== Token Analysis for simple_duplicate.py ===");
        println!("Total tokens: {}", tokens.len());

        // Count tokens by type
        let mut keyword_count = 0;
        let mut identifier_count = 0;
        let mut literal_count = 0;
        let mut whitespace_count = 0;

        for token in &tokens {
            match token.token_type {
                TokenType::Keyword => keyword_count += 1,
                TokenType::Identifier => identifier_count += 1,
                TokenType::Literal => literal_count += 1,
                TokenType::Whitespace => whitespace_count += 1,
                _ => {}
            }
        }

        println!("Keyword tokens: {}", keyword_count);
        println!("Identifier tokens: {}", identifier_count);
        println!("Literal tokens: {}", literal_count);
        println!("Whitespace tokens: {}", whitespace_count);

        // Normalized count (non-whitespace, non-comment)
        let normalized_count = tokens
            .iter()
            .filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::Comment))
            .count();

        println!("Normalized token count: {}", normalized_count);
        println!(
            "Meets min_tokens=50? {}",
            if normalized_count >= 50 { "YES" } else { "NO" }
        );

        // Should have enough tokens to potentially detect duplicates
        assert!(
            tokens.len() > 20,
            "Should have at least 20 tokens for two simple functions"
        );
    }

    #[test]
    fn test_module_docstring_detected_as_comment() {
        let tokenizer = PythonTokenizer::new();
        let source = r#""""Module-level docstring"""
def func():
    pass
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the module docstring - should be classified as Comment
        let docstring = tokens
            .iter()
            .find(|t| t.value.contains("Module-level docstring"));
        assert!(docstring.is_some(), "Should find the module docstring");
        assert_eq!(
            docstring.unwrap().token_type,
            TokenType::Comment,
            "Module docstring should be classified as Comment"
        );
    }

    #[test]
    fn test_function_docstring_detected_as_comment() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"def add(a, b):
    """Add two numbers together"""
    return a + b
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the function docstring - should be classified as Comment
        let docstring = tokens
            .iter()
            .find(|t| t.value.contains("Add two numbers together"));
        assert!(docstring.is_some(), "Should find the function docstring");
        assert_eq!(
            docstring.unwrap().token_type,
            TokenType::Comment,
            "Function docstring should be classified as Comment"
        );
    }

    #[test]
    fn test_class_docstring_detected_as_comment() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"class Calculator:
    """A simple calculator class"""
    def __init__(self):
        pass
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the class docstring - should be classified as Comment
        let docstring = tokens
            .iter()
            .find(|t| t.value.contains("simple calculator class"));
        assert!(docstring.is_some(), "Should find the class docstring");
        assert_eq!(
            docstring.unwrap().token_type,
            TokenType::Comment,
            "Class docstring should be classified as Comment"
        );
    }

    #[test]
    fn test_method_docstring_detected_as_comment() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"class Calculator:
    def add(self, a, b):
        """Add two numbers"""
        return a + b
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the method docstring - should be classified as Comment
        let docstring = tokens
            .iter()
            .find(|t| t.value.contains("Add two numbers"));
        assert!(docstring.is_some(), "Should find the method docstring");
        assert_eq!(
            docstring.unwrap().token_type,
            TokenType::Comment,
            "Method docstring should be classified as Comment"
        );
    }

    #[test]
    fn test_non_docstring_strings_remain_literals() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"def greet(name):
    """Greet someone"""
    message = "Hello, " + name
    return message
"#;

        let result = tokenizer.tokenize_python(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Find the docstring - should be Comment
        let docstring = tokens
            .iter()
            .find(|t| t.value.contains("Greet someone"));
        assert!(docstring.is_some());
        assert_eq!(
            docstring.unwrap().token_type,
            TokenType::Comment,
            "Docstring should be Comment"
        );

        // Find the regular string "Hello, " - should be Literal
        let regular_string = tokens.iter().find(|t| t.value.contains("Hello, "));
        assert!(regular_string.is_some(), "Should find regular string");
        assert_eq!(
            regular_string.unwrap().token_type,
            TokenType::Literal,
            "Regular string should remain as Literal"
        );
    }
}
