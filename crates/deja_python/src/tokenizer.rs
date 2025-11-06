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

    /// Tokenize Python source code
    fn tokenize_python(&self, source: &str) -> Result<Vec<Token>, TokenizationError> {
        let tokens: Vec<LexResult> = lex(source, Mode::Module).collect();

        let mut result = Vec::new();
        let mut offset = 0;

        for token_result in tokens {
            match token_result {
                Ok((loc, tok, end_loc)) => {
                    let start_line = loc.row().get();
                    let start_col = loc.column().get();
                    let end_line = end_loc.row().get();

                    let token_type = self.map_token_type(&tok);
                    let value = self.token_to_string(&tok);

                    let start_offset = offset;
                    let end_offset = offset + value.len();

                    result.push(Token::new(
                        token_type,
                        value,
                        start_offset,
                        end_offset,
                        start_line,
                        start_col,
                    ));

                    offset = end_offset;
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
            Tok::Comment(s) => s.to_string(),
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
            | Tok::CircumFlexEqual
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

            // Comments
            Tok::Comment(_) => TokenType::Comment,

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
}
