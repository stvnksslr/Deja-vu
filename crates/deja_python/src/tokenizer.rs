//! Python tokenizer for token-based clone detection

use deja_core::token::{Token, TokenType};
use ruff_python_parser::lexer::{lex, LexResult, LexicalError};
use ruff_python_parser::Mode;

/// Python tokenizer
pub struct PythonTokenizer;

impl PythonTokenizer {
    pub fn new() -> Self {
        Self
    }

    /// Tokenize Python source code
    pub fn tokenize(&self, source: &str) -> Result<Vec<Token>, TokenizationError> {
        let tokens: Vec<LexResult> = lex(source, Mode::Module).collect();

        let mut result = Vec::new();
        let mut line = 1;
        let mut col = 0;

        for (i, token_result) in tokens.iter().enumerate() {
            match token_result {
                Ok((tok, range)) => {
                    let start = range.start().to_usize();
                    let end = range.end().to_usize();

                    let token_type = self.map_token_type(tok);
                    let value = source[start..end].to_string();

                    // Update line and column tracking
                    let newlines = value.matches('\n').count();
                    if newlines > 0 {
                        line += newlines;
                        col = 0;
                    } else {
                        col += value.len();
                    }

                    result.push(Token::new(token_type, value, start, end, line, col));
                }
                Err(e) => {
                    return Err(TokenizationError::LexError(format!("{:?}", e)));
                }
            }
        }

        Ok(result)
    }

    fn map_token_type(&self, tok: &ruff_python_parser::Tok) -> TokenType {
        use ruff_python_parser::Tok;

        match tok {
            // Keywords
            Tok::And | Tok::Or | Tok::Not | Tok::Is | Tok::In | Tok::If | Tok::Else
            | Tok::Elif | Tok::While | Tok::For | Tok::Def | Tok::Class | Tok::Return
            | Tok::Yield | Tok::Import | Tok::From | Tok::As | Tok::Try | Tok::Except
            | Tok::Finally | Tok::With | Tok::Lambda | Tok::Pass | Tok::Break
            | Tok::Continue | Tok::Global | Tok::Nonlocal | Tok::Assert | Tok::Del
            | Tok::Raise | Tok::Async | Tok::Await | Tok::Match | Tok::Case | Tok::Type => {
                TokenType::Keyword
            }

            // Identifiers
            Tok::Name { .. } => TokenType::Identifier,

            // Literals
            Tok::Int { .. } | Tok::Float { .. } | Tok::Complex { .. } | Tok::String { .. }
            | Tok::FStringStart { .. } | Tok::FStringMiddle { .. } | Tok::FStringEnd { .. }
            | Tok::True | Tok::False | Tok::None => TokenType::Literal,

            // Operators
            Tok::Plus | Tok::Minus | Tok::Star | Tok::Slash | Tok::DoubleSlash | Tok::Percent
            | Tok::DoubleStar | Tok::Vbar | Tok::Amper | Tok::CircumFlex | Tok::LeftShift
            | Tok::RightShift | Tok::Less | Tok::Greater | Tok::LessEqual | Tok::GreaterEqual
            | Tok::EqEqual | Tok::NotEqual | Tok::Equal | Tok::PlusEqual | Tok::MinusEqual
            | Tok::StarEqual | Tok::SlashEqual | Tok::DoubleSlashEqual | Tok::PercentEqual
            | Tok::AmperEqual | Tok::VbarEqual | Tok::CircumFlexEqual | Tok::LeftShiftEqual
            | Tok::RightShiftEqual | Tok::DoubleStarEqual | Tok::Rarrow | Tok::ColonEqual
            | Tok::Dot | Tok::Ellipsis | Tok::At | Tok::AtEqual | Tok::Tilde => {
                TokenType::Operator
            }

            // Delimiters
            Tok::Lpar | Tok::Rpar | Tok::Lsqb | Tok::Rsqb | Tok::Lbrace | Tok::Rbrace
            | Tok::Comma | Tok::Colon | Tok::Semi => TokenType::Delimiter,

            // Comments
            Tok::Comment { .. } => TokenType::Comment,

            // Whitespace
            Tok::Newline | Tok::Indent | Tok::Dedent | Tok::NonLogicalNewline => {
                TokenType::Whitespace
            }

            // Unknown
            _ => TokenType::Unknown,
        }
    }
}

impl Default for PythonTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tokenization error
#[derive(Debug, thiserror::Error)]
pub enum TokenizationError {
    #[error("Lexical error: {0}")]
    LexError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_python() {
        let tokenizer = PythonTokenizer::new();
        let source = "def hello():\n    pass";

        let result = tokenizer.tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert!(!tokens.is_empty());

        // Should have 'def', 'hello', '(', ')', ':', newline, indent, 'pass', dedent
        let keyword_count = tokens.iter().filter(|t| t.token_type == TokenType::Keyword).count();
        assert!(keyword_count >= 2); // 'def' and 'pass'
    }

    #[test]
    fn test_tokenize_with_literals() {
        let tokenizer = PythonTokenizer::new();
        let source = r#"x = 42"#;

        let result = tokenizer.tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let literal_count = tokens.iter().filter(|t| t.token_type == TokenType::Literal).count();
        assert!(literal_count >= 1); // At least the number 42
    }
}
