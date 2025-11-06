//! Python AST to Generic AST converter
//!
//! This module converts Ruff's Python AST to our generic AST representation
//! for language-agnostic clone detection.

use deja_ast::{Ast, AstNode, NodeKind, Span, ToGenericAst, AstError};
use ruff_python_ast as ast;
use ruff_python_parser::{parse, Mode};
use ruff_text_size::TextSize;

/// Converts Python source code to a generic AST
pub fn parse_python_to_generic_ast(source: &str, source_file: &str) -> Result<Ast, AstError> {
    // Parse the Python source code
    let parsed = parse(source, Mode::Module)
        .map_err(|e| AstError::ParseError(format!("Failed to parse Python: {:?}", e)))?;

    // Convert to generic AST
    let mut converter = PythonAstConverter::new(source, source_file);
    converter.convert(&parsed)
}

/// Converter from Ruff Python AST to Generic AST
struct PythonAstConverter<'a> {
    source: &'a str,
    source_file: &'a str,
    ast: Ast,
    next_id: usize,
}

impl<'a> PythonAstConverter<'a> {
    fn new(source: &'a str, source_file: &'a str) -> Self {
        Self {
            source,
            source_file,
            ast: Ast::new(source_file.to_string()),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn create_span(&self, range: ruff_text_size::TextRange) -> Span {
        let start = range.start().to_usize();
        let end = range.end().to_usize();

        // Calculate line and column numbers
        let (line_start, column_start) = self.offset_to_position(start);
        let (line_end, column_end) = self.offset_to_position(end);

        Span::new(start, end, line_start, line_end, column_start, column_end)
    }

    fn offset_to_position(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 0;
        let mut current_offset = 0;

        for ch in self.source.chars() {
            if current_offset >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            current_offset += ch.len_utf8();
        }

        (line, col)
    }

    fn convert(mut self, module: &ast::Mod) -> Result<Ast, AstError> {
        match module {
            ast::Mod::Module(m) => {
                let module_id = self.next_id();
                let module_node = AstNode::new(
                    module_id,
                    NodeKind::Module,
                    Span::new(0, self.source.len(), 1, self.source.lines().count(), 0, 0),
                );

                self.ast.add_node(module_node);
                self.ast.root = module_id;

                // Convert all statements in the module
                let mut children = Vec::new();
                for stmt in &m.body {
                    if let Some(child_id) = self.convert_stmt(stmt) {
                        children.push(child_id);
                    }
                }

                // Update module node with children
                if let Some(node) = self.ast.get_node_mut(module_id) {
                    node.children = children;
                }

                Ok(self.ast)
            }
            _ => Err(AstError::UnsupportedNode("Only Module mode supported".to_string())),
        }
    }

    fn convert_stmt(&mut self, stmt: &ast::Stmt) -> Option<usize> {
        let id = self.next_id();
        let span = self.create_span(stmt.range());

        let (kind, text, children) = match stmt {
            ast::Stmt::FunctionDef(func) => {
                let mut func_children = Vec::new();

                // Add parameters
                for param in &func.parameters.args {
                    if let Some(param_id) = self.convert_parameter(param) {
                        func_children.push(param_id);
                    }
                }

                // Add body statements
                for stmt in &func.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        func_children.push(stmt_id);
                    }
                }

                (NodeKind::Function, Some(func.name.to_string()), func_children)
            }

            ast::Stmt::ClassDef(class) => {
                let mut class_children = Vec::new();
                for stmt in &class.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        class_children.push(stmt_id);
                    }
                }
                (NodeKind::Class, Some(class.name.to_string()), class_children)
            }

            ast::Stmt::Return(ret) => {
                let mut children = Vec::new();
                if let Some(value) = &ret.value {
                    if let Some(expr_id) = self.convert_expr(value) {
                        children.push(expr_id);
                    }
                }
                (NodeKind::Return, None, children)
            }

            ast::Stmt::Assign(assign) => {
                let mut children = Vec::new();
                if let Some(value_id) = self.convert_expr(&assign.value) {
                    children.push(value_id);
                }
                (NodeKind::Assignment, None, children)
            }

            ast::Stmt::AugAssign(_) => (NodeKind::AugmentedAssignment, None, Vec::new()),

            ast::Stmt::If(if_stmt) => {
                let mut children = Vec::new();

                // Add body statements
                for stmt in &if_stmt.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        children.push(stmt_id);
                    }
                }

                // Add elif/else statements
                for stmt in &if_stmt.elif_else_clauses {
                    if let Some(stmt_id) = self.convert_stmt(&stmt.body[0]) {
                        children.push(stmt_id);
                    }
                }

                (NodeKind::IfStatement, None, children)
            }

            ast::Stmt::For(for_loop) => {
                let mut children = Vec::new();
                for stmt in &for_loop.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        children.push(stmt_id);
                    }
                }
                (NodeKind::ForLoop, None, children)
            }

            ast::Stmt::While(while_loop) => {
                let mut children = Vec::new();
                for stmt in &while_loop.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        children.push(stmt_id);
                    }
                }
                (NodeKind::WhileLoop, None, children)
            }

            ast::Stmt::Try(try_stmt) => {
                let mut children = Vec::new();
                for stmt in &try_stmt.body {
                    if let Some(stmt_id) = self.convert_stmt(stmt) {
                        children.push(stmt_id);
                    }
                }
                (NodeKind::TryStatement, None, children)
            }

            ast::Stmt::With(_) => (NodeKind::WithStatement, None, Vec::new()),
            ast::Stmt::Break(_) => (NodeKind::Break, None, Vec::new()),
            ast::Stmt::Continue(_) => (NodeKind::Continue, None, Vec::new()),
            ast::Stmt::Pass(_) => (NodeKind::Pass, None, Vec::new()),
            ast::Stmt::Raise(_) => (NodeKind::Raise, None, Vec::new()),
            ast::Stmt::Assert(_) => (NodeKind::Assert, None, Vec::new()),
            ast::Stmt::Import(_) => (NodeKind::Import, None, Vec::new()),
            ast::Stmt::ImportFrom(_) => (NodeKind::ImportFrom, None, Vec::new()),

            ast::Stmt::Expr(expr) => {
                let mut children = Vec::new();
                if let Some(expr_id) = self.convert_expr(&expr.value) {
                    children.push(expr_id);
                }
                (NodeKind::Expression, None, children)
            }

            _ => (NodeKind::Unknown("UnsupportedStatement".to_string()), None, Vec::new()),
        };

        let mut node = AstNode::new(id, kind, span);
        if let Some(text) = text {
            node = node.with_text(text);
        }
        node = node.with_children(children);

        self.ast.add_node(node);
        Some(id)
    }

    fn convert_parameter(&mut self, param: &ast::Parameter) -> Option<usize> {
        let id = self.next_id();
        let span = self.create_span(param.range);

        let node = AstNode::new(id, NodeKind::Parameter, span)
            .with_text(param.name.to_string());

        self.ast.add_node(node);
        Some(id)
    }

    fn convert_expr(&mut self, expr: &ast::Expr) -> Option<usize> {
        let id = self.next_id();
        let span = self.create_span(expr.range());

        let (kind, text, children) = match expr {
            ast::Expr::Name(name) => {
                (NodeKind::Identifier, Some(name.id.to_string()), Vec::new())
            }

            ast::Expr::NumberLiteral(_) => (NodeKind::Literal, Some("$NUM".to_string()), Vec::new()),
            ast::Expr::StringLiteral(_) => (NodeKind::Literal, Some("$STR".to_string()), Vec::new()),
            ast::Expr::BytesLiteral(_) => (NodeKind::Literal, Some("$BYTES".to_string()), Vec::new()),
            ast::Expr::BooleanLiteral(_) => (NodeKind::Literal, Some("$BOOL".to_string()), Vec::new()),
            ast::Expr::NoneLiteral(_) => (NodeKind::Literal, Some("$NONE".to_string()), Vec::new()),

            ast::Expr::BinOp(binop) => {
                let mut children = Vec::new();
                if let Some(left_id) = self.convert_expr(&binop.left) {
                    children.push(left_id);
                }
                if let Some(right_id) = self.convert_expr(&binop.right) {
                    children.push(right_id);
                }
                (NodeKind::BinaryOp, Some(format!("{:?}", binop.op)), children)
            }

            ast::Expr::UnaryOp(unop) => {
                let mut children = Vec::new();
                if let Some(operand_id) = self.convert_expr(&unop.operand) {
                    children.push(operand_id);
                }
                (NodeKind::UnaryOp, Some(format!("{:?}", unop.op)), children)
            }

            ast::Expr::Compare(comp) => {
                let mut children = Vec::new();
                if let Some(left_id) = self.convert_expr(&comp.left) {
                    children.push(left_id);
                }
                for comparator in &comp.comparators {
                    if let Some(comp_id) = self.convert_expr(comparator) {
                        children.push(comp_id);
                    }
                }
                (NodeKind::CompareOp, None, children)
            }

            ast::Expr::BoolOp(boolop) => {
                let mut children = Vec::new();
                for value in &boolop.values {
                    if let Some(value_id) = self.convert_expr(value) {
                        children.push(value_id);
                    }
                }
                (NodeKind::BoolOp, Some(format!("{:?}", boolop.op)), children)
            }

            ast::Expr::Call(call) => {
                let mut children = Vec::new();
                if let Some(func_id) = self.convert_expr(&call.func) {
                    children.push(func_id);
                }
                for arg in &call.arguments.args {
                    if let Some(arg_id) = self.convert_expr(arg) {
                        children.push(arg_id);
                    }
                }
                (NodeKind::FunctionCall, None, children)
            }

            ast::Expr::Attribute(attr) => {
                let mut children = Vec::new();
                if let Some(value_id) = self.convert_expr(&attr.value) {
                    children.push(value_id);
                }
                (NodeKind::Attribute, Some(attr.attr.to_string()), children)
            }

            ast::Expr::Subscript(subscript) => {
                let mut children = Vec::new();
                if let Some(value_id) = self.convert_expr(&subscript.value) {
                    children.push(value_id);
                }
                (NodeKind::Subscript, None, children)
            }

            ast::Expr::List(_) => (NodeKind::ListLiteral, None, Vec::new()),
            ast::Expr::Tuple(_) => (NodeKind::TupleLiteral, None, Vec::new()),
            ast::Expr::Set(_) => (NodeKind::SetLiteral, None, Vec::new()),
            ast::Expr::Dict(_) => (NodeKind::DictLiteral, None, Vec::new()),

            ast::Expr::Lambda(_) => (NodeKind::Lambda, None, Vec::new()),
            ast::Expr::ListComp(_) => (NodeKind::ListComp, None, Vec::new()),
            ast::Expr::DictComp(_) => (NodeKind::DictComp, None, Vec::new()),
            ast::Expr::SetComp(_) => (NodeKind::SetComp, None, Vec::new()),

            _ => (NodeKind::Unknown("UnsupportedExpression".to_string()), None, Vec::new()),
        };

        let mut node = AstNode::new(id, kind, span);
        if let Some(text) = text {
            node = node.with_text(text);
        }
        node = node.with_children(children);

        self.ast.add_node(node);
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
def add(a, b):
    return a + b
"#;

        let ast = parse_python_to_generic_ast(source, "test.py").unwrap();
        assert_eq!(ast.nodes.len() > 0, true);

        // Root should be a module
        let root = ast.get_node(ast.root).unwrap();
        assert_eq!(root.kind, NodeKind::Module);

        // Should have at least one child (the function)
        assert!(root.children.len() > 0);
    }

    #[test]
    fn test_parse_class() {
        let source = r#"
class Calculator:
    def add(self, a, b):
        return a + b
"#;

        let ast = parse_python_to_generic_ast(source, "test.py").unwrap();

        let root = ast.get_node(ast.root).unwrap();
        assert!(root.children.len() > 0);

        // First child should be a class
        let class_node = ast.get_node(root.children[0]).unwrap();
        assert_eq!(class_node.kind, NodeKind::Class);
        assert_eq!(class_node.text, Some("Calculator".to_string()));
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
if x > 0:
    print("positive")
else:
    print("negative")
"#;

        let ast = parse_python_to_generic_ast(source, "test.py").unwrap();
        let root = ast.get_node(ast.root).unwrap();

        // Should have an if statement
        let if_node = ast.get_node(root.children[0]).unwrap();
        assert_eq!(if_node.kind, NodeKind::IfStatement);
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "def invalid syntax here";
        let result = parse_python_to_generic_ast(source, "test.py");
        assert!(result.is_err());
    }
}
