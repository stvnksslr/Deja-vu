//! Token-based representation for fast clone detection

use serde::{Deserialize, Serialize};

/// Token type for lexical analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    Identifier,
    Literal,
    Operator,
    Delimiter,
    Comment,
    Whitespace,
    Unknown,
}

/// A token from lexical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        value: String,
        start: usize,
        end: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            token_type,
            value,
            start,
            end,
            line,
            column,
        }
    }

    /// Returns a normalized version of this token for comparison
    pub fn normalize(&self, ignore_identifiers: bool) -> String {
        match self.token_type {
            TokenType::Identifier if ignore_identifiers => "$ID".to_string(),
            TokenType::Literal => "$LIT".to_string(),
            TokenType::Comment | TokenType::Whitespace => String::new(),
            _ => self.value.clone(),
        }
    }
}

/// A sequence of tokens representing a code fragment
#[derive(Debug, Clone)]
pub struct TokenSequence {
    pub tokens: Vec<Token>,
    pub start_line: usize,
    pub end_line: usize,
}

impl TokenSequence {
    pub fn new(tokens: Vec<Token>) -> Self {
        let start_line = tokens.first().map(|t| t.line).unwrap_or(0);
        let end_line = tokens.last().map(|t| t.line).unwrap_or(0);

        Self {
            tokens,
            start_line,
            end_line,
        }
    }

    /// Returns the number of significant tokens (excluding whitespace/comments)
    pub fn significant_token_count(&self) -> usize {
        self.tokens
            .iter()
            .filter(|t| !matches!(t.token_type, TokenType::Comment | TokenType::Whitespace))
            .count()
    }

    /// Returns a normalized string representation for hashing
    pub fn to_normalized_string(&self, ignore_identifiers: bool) -> String {
        self.tokens
            .iter()
            .map(|t| t.normalize(ignore_identifiers))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_normalize() {
        let token = Token::new(TokenType::Identifier, "foo".to_string(), 0, 3, 1, 0);
        assert_eq!(token.normalize(true), "$ID");
        assert_eq!(token.normalize(false), "foo");
    }

    #[test]
    fn test_token_sequence() {
        let tokens = vec![
            Token::new(TokenType::Keyword, "def".to_string(), 0, 3, 1, 0),
            Token::new(TokenType::Identifier, "foo".to_string(), 4, 7, 1, 4),
        ];

        let seq = TokenSequence::new(tokens);
        assert_eq!(seq.significant_token_count(), 2);
        assert_eq!(seq.to_normalized_string(true), "def $ID");
    }
}
