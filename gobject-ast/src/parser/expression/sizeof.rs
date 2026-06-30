use tree_sitter::Node;

use crate::{
    model::{SizeofExpression, SizeofOperand, TypeInfo},
    parser::Parser,
};

impl Parser {
    pub(crate) fn parse_sizeof_expression(
        &self,
        node: Node,
        source: &[u8],
    ) -> Option<SizeofExpression> {
        let text = std::str::from_utf8(&source[node.byte_range()])
            .ok()?
            .to_owned();

        let mut operand = None;

        // Walk children to find what sizeof is operating on
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "sizeof" | "(" | ")" => continue,

                "type_descriptor" => {
                    let type_text = std::str::from_utf8(&source[child.byte_range()]).ok()?;
                    let type_info = TypeInfo::new(type_text, self.node_location(child));
                    operand = Some(SizeofOperand::Type(type_info));
                }

                // sizeof(X) where X is an identifier is ambiguous — tree-sitter
                // defaults to expression, but X may be a typedef'd type.  Check
                // the pre-scanned known_types to resolve the ambiguity.
                "parenthesized_expression"
                    if child.named_child_count() == 1
                        && let Some(id_node) = child.named_child(0)
                        && id_node.kind() == "identifier"
                        && let Ok(id_text) = std::str::from_utf8(&source[id_node.byte_range()])
                        && self.known_types.contains(id_text) =>
                {
                    let type_info = TypeInfo::new(id_text, self.node_location(id_node));
                    operand = Some(SizeofOperand::Type(type_info));
                }

                _ if child.is_named() && Self::is_expression_node(&child) => {
                    if let Some(expr) = self.parse_expression(child, source) {
                        operand = Some(SizeofOperand::Expression(Box::new(expr)));
                    }
                }
                _ => {}
            }
        }

        Some(SizeofExpression {
            operand,
            text,
            location: self.node_location(node),
        })
    }
}
