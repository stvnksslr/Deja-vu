//! Language-agnostic AST representation for code clone detection
//!
//! This crate provides generic abstractions for representing Abstract Syntax Trees
//! from various programming languages in a unified way, enabling language-agnostic
//! clone detection algorithms.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A unique identifier for a node in the AST
pub type NodeId = usize;

/// Location information for a code span
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl Span {
    pub fn new(
        start: usize,
        end: usize,
        line_start: usize,
        line_end: usize,
        column_start: usize,
        column_end: usize,
    ) -> Self {
        Self {
            start,
            end,
            line_start,
            line_end,
            column_start,
            column_end,
        }
    }

    /// Returns the length of the span in bytes
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Generic node type for language-agnostic AST representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    // Structural nodes
    Module,
    Class,
    Function,
    Method,
    Block,
    Parameter,

    // Statement nodes
    IfStatement,
    ElifStatement,
    ElseStatement,
    WhileLoop,
    ForLoop,
    TryStatement,
    ExceptHandler,
    FinallyStatement,
    WithStatement,
    Return,
    Break,
    Continue,
    Pass,
    Raise,
    Assert,
    Assignment,
    AugmentedAssignment,
    Expression,
    Import,
    ImportFrom,

    // Expression nodes
    BinaryOp,
    UnaryOp,
    CompareOp,
    BoolOp,
    FunctionCall,
    MethodCall,
    Attribute,
    Subscript,
    Identifier,
    Literal,
    ListLiteral,
    DictLiteral,
    SetLiteral,
    TupleLiteral,
    Lambda,
    ListComp,
    DictComp,
    SetComp,

    // Other
    Comment,
    Docstring,
    Decorator,
    Unknown(String),
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Module => write!(f, "Module"),
            NodeKind::Class => write!(f, "Class"),
            NodeKind::Function => write!(f, "Function"),
            NodeKind::Method => write!(f, "Method"),
            NodeKind::Block => write!(f, "Block"),
            NodeKind::Parameter => write!(f, "Parameter"),
            NodeKind::IfStatement => write!(f, "IfStatement"),
            NodeKind::ElifStatement => write!(f, "ElifStatement"),
            NodeKind::ElseStatement => write!(f, "ElseStatement"),
            NodeKind::WhileLoop => write!(f, "WhileLoop"),
            NodeKind::ForLoop => write!(f, "ForLoop"),
            NodeKind::TryStatement => write!(f, "TryStatement"),
            NodeKind::ExceptHandler => write!(f, "ExceptHandler"),
            NodeKind::FinallyStatement => write!(f, "FinallyStatement"),
            NodeKind::WithStatement => write!(f, "WithStatement"),
            NodeKind::Return => write!(f, "Return"),
            NodeKind::Break => write!(f, "Break"),
            NodeKind::Continue => write!(f, "Continue"),
            NodeKind::Pass => write!(f, "Pass"),
            NodeKind::Raise => write!(f, "Raise"),
            NodeKind::Assert => write!(f, "Assert"),
            NodeKind::Assignment => write!(f, "Assignment"),
            NodeKind::AugmentedAssignment => write!(f, "AugmentedAssignment"),
            NodeKind::Expression => write!(f, "Expression"),
            NodeKind::Import => write!(f, "Import"),
            NodeKind::ImportFrom => write!(f, "ImportFrom"),
            NodeKind::BinaryOp => write!(f, "BinaryOp"),
            NodeKind::UnaryOp => write!(f, "UnaryOp"),
            NodeKind::CompareOp => write!(f, "CompareOp"),
            NodeKind::BoolOp => write!(f, "BoolOp"),
            NodeKind::FunctionCall => write!(f, "FunctionCall"),
            NodeKind::MethodCall => write!(f, "MethodCall"),
            NodeKind::Attribute => write!(f, "Attribute"),
            NodeKind::Subscript => write!(f, "Subscript"),
            NodeKind::Identifier => write!(f, "Identifier"),
            NodeKind::Literal => write!(f, "Literal"),
            NodeKind::ListLiteral => write!(f, "ListLiteral"),
            NodeKind::DictLiteral => write!(f, "DictLiteral"),
            NodeKind::SetLiteral => write!(f, "SetLiteral"),
            NodeKind::TupleLiteral => write!(f, "TupleLiteral"),
            NodeKind::Lambda => write!(f, "Lambda"),
            NodeKind::ListComp => write!(f, "ListComp"),
            NodeKind::DictComp => write!(f, "DictComp"),
            NodeKind::SetComp => write!(f, "SetComp"),
            NodeKind::Comment => write!(f, "Comment"),
            NodeKind::Docstring => write!(f, "Docstring"),
            NodeKind::Decorator => write!(f, "Decorator"),
            NodeKind::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

/// A node in the generic AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub span: Span,
    pub children: Vec<NodeId>,
    /// Optional text content (for identifiers, literals, etc.)
    pub text: Option<String>,
}

impl AstNode {
    pub fn new(id: NodeId, kind: NodeKind, span: Span) -> Self {
        Self {
            id,
            kind,
            span,
            children: Vec::new(),
            text: None,
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn with_children(mut self, children: Vec<NodeId>) -> Self {
        self.children = children;
        self
    }
}

/// A generic Abstract Syntax Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ast {
    pub nodes: Vec<AstNode>,
    pub root: NodeId,
    pub source_file: String,
}

impl Ast {
    pub fn new(source_file: String) -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
            source_file,
        }
    }

    pub fn add_node(&mut self, node: AstNode) -> NodeId {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&AstNode> {
        // Search for node by ID, not by index
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut AstNode> {
        // Search for node by ID, not by index
        self.nodes.iter_mut().find(|node| node.id == id)
    }
}

/// Trait for converting language-specific ASTs to the generic representation
pub trait ToGenericAst {
    fn to_generic_ast(&self, source_file: &str) -> Result<Ast, AstError>;
}

/// Error type for AST operations
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("Failed to parse source: {0}")]
    ParseError(String),

    #[error("Invalid AST structure: {0}")]
    InvalidStructure(String),

    #[error("Unsupported node type: {0}")]
    UnsupportedNode(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 10, 1, 1, 0, 10);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_ast_node_creation() {
        let span = Span::new(0, 5, 1, 1, 0, 5);
        let node = AstNode::new(0, NodeKind::Function, span).with_text("my_function".to_string());

        assert_eq!(node.id, 0);
        assert_eq!(node.kind, NodeKind::Function);
        assert_eq!(node.text, Some("my_function".to_string()));
    }
}
