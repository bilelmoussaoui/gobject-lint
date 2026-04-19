use tree_sitter::Node;

use crate::{model::SwitchStatement, parser::Parser};

impl Parser {
    pub(crate) fn parse_switch_statement(
        &self,
        node: Node,
        source: &[u8],
    ) -> Option<SwitchStatement> {
        let condition_node = node.child(1)?; // parenthesized_expression
        // Parse the expression inside the parentheses
        let inner_expr = condition_node.child(1).or(Some(condition_node))?;
        let condition = self.parse_expression(inner_expr, source)?;
        let condition_location = self.node_location(inner_expr);

        let body_node = node.child(2)?; // compound_statement
        let body = self.parse_function_body(body_node, source);

        Some(SwitchStatement {
            condition,
            condition_location,
            body,
            location: self.node_location(node),
        })
    }
}
