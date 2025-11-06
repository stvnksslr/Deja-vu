//! Python AST Parser implementing the LanguageParser trait

use deja_ast::Ast;
use deja_core::LanguageParser;
use anyhow::Result;
use crate::ast_converter::parse_python_to_generic_ast;

/// Python parser for AST-based detection
pub struct PythonAstParser;

impl PythonAstParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageParser for PythonAstParser {
    fn parse(&self, source: &str, source_file: &str) -> Result<Ast> {
        parse_python_to_generic_ast(source, source_file)
            .map_err(|e| anyhow::anyhow!("Failed to parse Python: {}", e))
    }

    fn language(&self) -> &str {
        "python"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_ast_parser() {
        let parser = PythonAstParser::new();
        assert_eq!(parser.language(), "python");

        let source = r#"
def add(a, b):
    return a + b
"#;

        let ast = parser.parse(source, "test.py");
        assert!(ast.is_ok());
    }
}
