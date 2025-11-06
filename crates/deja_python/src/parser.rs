//! Python AST parser using Ruff's Python parser

use deja_ast::{Ast, AstError, AstNode, NodeKind, Span, ToGenericAst};
use ruff_python_ast::Mod;
use ruff_python_parser::{parse, Mode};
use ruff_text_size::TextSize;

/// Python language parser
pub struct PythonParser;

impl PythonParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse Python source code into an AST
    pub fn parse_source(&self, source: &str, filename: &str) -> Result<Ast, AstError> {
        let parsed = parse(source, Mode::Module, filename)
            .map_err(|e| AstError::ParseError(format!("Failed to parse Python: {:?}", e)))?;

        self.convert_to_generic_ast(parsed, source, filename)
    }

    fn convert_to_generic_ast(
        &self,
        module: Mod,
        _source: &str,
        filename: &str,
    ) -> Result<Ast, AstError> {
        let mut ast = Ast::new(filename.to_string());

        match module {
            Mod::Module(mod_ast) => {
                let root_span = Span::new(0, 0, 0, 0, 0, 0);
                let root_node = AstNode::new(0, NodeKind::Module, root_span);
                ast.add_node(root_node);
                ast.root = 0;

                // TODO: Walk the Python AST and convert to generic AST
                // This is a simplified implementation - full implementation would
                // recursively process all statements and expressions

                Ok(ast)
            }
            Mod::Expression(_) => {
                Err(AstError::UnsupportedNode("Expression mode not supported".to_string()))
            }
        }
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ToGenericAst for PythonParser {
    fn to_generic_ast(&self, source_file: &str) -> Result<Ast, AstError> {
        // Read file and parse
        let content = std::fs::read_to_string(source_file)
            .map_err(|e| AstError::ParseError(format!("Failed to read file: {}", e)))?;

        self.parse_source(&content, source_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_python() {
        let parser = PythonParser::new();
        let source = r#"
def hello():
    print("Hello, world!")
"#;

        let result = parser.parse_source(source, "test.py");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_python() {
        let parser = PythonParser::new();
        let source = "def invalid syntax here";

        let result = parser.parse_source(source, "test.py");
        assert!(result.is_err());
    }
}
