//! Tree Edit Distance Algorithm
//!
//! Implements a simplified tree edit distance algorithm for comparing AST subtrees.
//! This is used to determine the similarity between code blocks at the structural level.
//!
//! The algorithm calculates the minimum cost to transform one tree into another using:
//! - Insert: Add a new node
//! - Delete: Remove a node
//! - Update: Change a node's label

use deja_ast::{Ast, AstNode, NodeKind};

/// Cost weights for different edit operations
#[derive(Debug, Clone)]
pub struct EditCosts {
    pub insert: f64,
    pub delete: f64,
    pub update: f64,
}

impl Default for EditCosts {
    fn default() -> Self {
        Self {
            insert: 1.0,
            delete: 1.0,
            update: 1.0,
        }
    }
}

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 100;

/// Maximum tree size to compare (prevents comparing huge subtrees)
const MAX_TREE_SIZE: usize = 500;

/// Calculates the tree edit distance between two AST subtrees
pub fn tree_edit_distance(
    ast1: &Ast,
    node1_id: usize,
    ast2: &Ast,
    node2_id: usize,
    costs: &EditCosts,
) -> f64 {
    tree_edit_distance_with_depth(ast1, node1_id, ast2, node2_id, costs, 0)
}

/// Internal function with depth tracking to prevent stack overflow
fn tree_edit_distance_with_depth(
    ast1: &Ast,
    node1_id: usize,
    ast2: &Ast,
    node2_id: usize,
    costs: &EditCosts,
    depth: usize,
) -> f64 {
    // Prevent stack overflow by limiting recursion depth
    if depth > MAX_RECURSION_DEPTH {
        // Return a large distance to indicate these trees are too different to compare safely
        return 1000.0;
    }

    let node1 = ast1.get_node(node1_id);
    let node2 = ast2.get_node(node2_id);

    if node1.is_none() && node2.is_none() {
        return 0.0;
    }

    if node1.is_none() {
        // Need to insert all nodes in tree2
        let size = count_nodes(ast2, node2_id);
        if size > MAX_TREE_SIZE {
            return 1000.0; // Too large, bail out
        }
        return size as f64 * costs.insert;
    }

    if node2.is_none() {
        // Need to delete all nodes in tree1
        let size = count_nodes(ast1, node1_id);
        if size > MAX_TREE_SIZE {
            return 1000.0; // Too large, bail out
        }
        return size as f64 * costs.delete;
    }

    let node1 = node1.unwrap();
    let node2 = node2.unwrap();

    // Use dynamic programming to compute edit distance
    compute_edit_distance_dp(ast1, node1, ast2, node2, costs, depth)
}

/// Compute edit distance using dynamic programming
fn compute_edit_distance_dp(
    ast1: &Ast,
    node1: &AstNode,
    ast2: &Ast,
    node2: &AstNode,
    costs: &EditCosts,
    depth: usize,
) -> f64 {
    let children1 = &node1.children;
    let children2 = &node2.children;

    // Cost of updating node labels
    let update_cost = if nodes_match(node1, node2) {
        0.0
    } else {
        costs.update
    };

    if children1.is_empty() && children2.is_empty() {
        return update_cost;
    }

    // Limit the number of children to compare to prevent explosion
    let max_children = 20;
    if children1.len() > max_children || children2.len() > max_children {
        // Too many children, use simplified comparison
        return update_cost + (children1.len().abs_diff(children2.len()) as f64 * costs.insert);
    }

    // If one tree has no children, calculate cost of inserting/deleting all children
    if children1.is_empty() {
        let insert_cost: f64 = children2.iter()
            .map(|&child_id| count_nodes(ast2, child_id) as f64 * costs.insert)
            .sum();
        return update_cost + insert_cost;
    }

    if children2.is_empty() {
        let delete_cost: f64 = children1.iter()
            .map(|&child_id| count_nodes(ast1, child_id) as f64 * costs.delete)
            .sum();
        return update_cost + delete_cost;
    }

    // Dynamic programming table for children alignment
    let m = children1.len();
    let n = children2.len();
    let mut dp = vec![vec![0.0; n + 1]; m + 1];

    // Initialize base cases
    for i in 0..=m {
        dp[i][0] = i as f64 * costs.delete;
    }
    for j in 0..=n {
        dp[0][j] = j as f64 * costs.insert;
    }

    // Fill DP table
    for i in 1..=m {
        for j in 1..=n {
            let child1_id = children1[i - 1];
            let child2_id = children2[j - 1];

            // Cost of updating child i to child j (recursive call with incremented depth)
            let match_cost = tree_edit_distance_with_depth(ast1, child1_id, ast2, child2_id, costs, depth + 1);

            // Minimum of: match/update, delete child1[i], insert child2[j]
            dp[i][j] = (dp[i - 1][j - 1] + match_cost)
                .min(dp[i - 1][j] + costs.delete)
                .min(dp[i][j - 1] + costs.insert);
        }
    }

    update_cost + dp[m][n]
}

/// Check if two nodes match (same kind and similar text)
fn nodes_match(node1: &AstNode, node2: &AstNode) -> bool {
    if node1.kind != node2.kind {
        return false;
    }

    // For structural nodes, kind match is enough
    match node1.kind {
        NodeKind::Module
        | NodeKind::Class
        | NodeKind::Function
        | NodeKind::Method
        | NodeKind::Block
        | NodeKind::IfStatement
        | NodeKind::ForLoop
        | NodeKind::WhileLoop => true,

        // For named nodes (identifiers, functions), check if text matches
        _ => node1.text == node2.text,
    }
}

/// Count total number of nodes in a subtree (iterative to avoid stack overflow)
fn count_nodes(ast: &Ast, node_id: usize) -> usize {
    let mut count = 0;
    let mut stack = vec![node_id];
    const MAX_NODES_TO_COUNT: usize = 1000; // Safety limit

    while let Some(current_id) = stack.pop() {
        if count >= MAX_NODES_TO_COUNT {
            return MAX_NODES_TO_COUNT; // Return limit if exceeded
        }

        if let Some(node) = ast.get_node(current_id) {
            count += 1;
            // Add all children to stack
            for &child_id in &node.children {
                stack.push(child_id);
            }
        }
    }

    count
}

/// Calculate similarity score from edit distance (0.0 = completely different, 1.0 = identical)
pub fn edit_distance_to_similarity(distance: f64, tree1_size: usize, tree2_size: usize) -> f64 {
    let max_size = tree1_size.max(tree2_size) as f64;

    if max_size == 0.0 {
        return 1.0;
    }

    // Similarity = 1 - (normalized_distance)
    (max_size - distance) / max_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use deja_ast::{Span, NodeKind, AstNode};

    fn create_test_ast() -> Ast {
        let mut ast = Ast::new("test.py".to_string());

        // Create a simple tree:
        //   Function
        //     ├─ Return
        //     │   └─ BinaryOp
        //     │       ├─ Identifier("a")
        //     │       └─ Identifier("b")

        let func_node = AstNode::new(
            0,
            NodeKind::Function,
            Span::new(0, 20, 1, 3, 0, 0),
        ).with_text("add".to_string());

        let return_node = AstNode::new(
            1,
            NodeKind::Return,
            Span::new(10, 20, 2, 2, 4, 14),
        );

        let binop_node = AstNode::new(
            2,
            NodeKind::BinaryOp,
            Span::new(17, 22, 2, 2, 11, 16),
        ).with_text("+".to_string());

        let id1_node = AstNode::new(
            3,
            NodeKind::Identifier,
            Span::new(17, 18, 2, 2, 11, 12),
        ).with_text("a".to_string());

        let id2_node = AstNode::new(
            4,
            NodeKind::Identifier,
            Span::new(21, 22, 2, 2, 15, 16),
        ).with_text("b".to_string());

        ast.add_node(func_node);
        ast.add_node(return_node);
        ast.add_node(binop_node);
        ast.add_node(id1_node);
        ast.add_node(id2_node);

        // Set up tree structure
        ast.get_node_mut(0).unwrap().children = vec![1];
        ast.get_node_mut(1).unwrap().children = vec![2];
        ast.get_node_mut(2).unwrap().children = vec![3, 4];

        ast.root = 0;
        ast
    }

    #[test]
    fn test_identical_trees() {
        let ast1 = create_test_ast();
        let ast2 = create_test_ast();

        let costs = EditCosts::default();
        let distance = tree_edit_distance(&ast1, 0, &ast2, 0, &costs);

        // Identical trees should have distance 0
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_count_nodes() {
        let ast = create_test_ast();

        // Root function node + return + binop + 2 identifiers = 5 nodes
        let count = count_nodes(&ast, 0);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_edit_distance_to_similarity() {
        // Perfect match: distance 0, size 5
        let sim = edit_distance_to_similarity(0.0, 5, 5);
        assert_eq!(sim, 1.0);

        // Half different: distance 2.5, size 5
        let sim = edit_distance_to_similarity(2.5, 5, 5);
        assert_eq!(sim, 0.5);

        // Completely different: distance 5, size 5
        let sim = edit_distance_to_similarity(5.0, 5, 5);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_nodes_match() {
        let node1 = AstNode::new(0, NodeKind::Function, Span::new(0, 10, 1, 1, 0, 10))
            .with_text("foo".to_string());

        let node2 = AstNode::new(1, NodeKind::Function, Span::new(0, 10, 1, 1, 0, 10))
            .with_text("bar".to_string());

        // Functions match by kind regardless of name (for structural matching)
        assert!(nodes_match(&node1, &node2));

        let id1 = AstNode::new(0, NodeKind::Identifier, Span::new(0, 1, 1, 1, 0, 1))
            .with_text("x".to_string());

        let id2 = AstNode::new(1, NodeKind::Identifier, Span::new(0, 1, 1, 1, 0, 1))
            .with_text("y".to_string());

        // Identifiers don't match if text differs
        assert!(!nodes_match(&id1, &id2));
    }
}
