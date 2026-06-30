use tree_sitter::Node;

use crate::{
    model::{AutoCleanupMacro, CompoundStatement, Statement, TypeInfo},
    parser::Parser,
};

impl Parser {
    pub(crate) fn parse_compound_statement(
        &self,
        node: Node,
        source: &[u8],
    ) -> Option<CompoundStatement> {
        let statements = self.parse_function_body(node, source);

        Some(CompoundStatement {
            statements,
            location: self.node_location(node),
        })
    }

    pub(crate) fn parse_function_body(&self, body_node: Node, source: &[u8]) -> Vec<Statement> {
        let mut statements = Vec::new();
        // tree-sitter splits `g_autofree struct Foo *var = NULL;` into two
        // declaration nodes: (1) `g_autofree struct` and (2) `Foo *var = NULL`.
        // The first fails to parse (no variable name), so we detect the
        // pattern and carry the struct/union + auto_cleanup to the next one.
        let mut pending_struct_fixup: Option<(bool, AutoCleanupMacro)> = None;

        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if let Some(mut stmt) = self.parse_statement(child, source) {
                if let Statement::Declaration(decl) = &mut stmt
                    && let Some((is_struct, auto_cleanup)) = pending_struct_fixup.take()
                {
                    if is_struct {
                        decl.type_info.is_struct = true;
                    } else {
                        decl.type_info.is_union = true;
                    }
                    decl.type_info.auto_cleanup = Some(auto_cleanup);
                }
                statements.push(stmt);
            } else if child.kind() == "declaration" {
                pending_struct_fixup = Self::detect_autofree_struct_stub(child, source);
            }
        }

        statements
    }

    /// Detect the pattern where tree-sitter split `g_autofree struct Foo *var`
    /// into a declaration `g_autofree struct` (which fails to parse).
    /// Returns `Some((is_struct, auto_cleanup))` when detected.
    fn detect_autofree_struct_stub(node: Node, source: &[u8]) -> Option<(bool, AutoCleanupMacro)> {
        let text = std::str::from_utf8(&source[node.byte_range()]).ok()?;
        let auto_cleanup = TypeInfo::parse_auto_cleanup(text)?;
        let trimmed = text.trim().trim_end_matches(';').trim();
        if trimmed.ends_with(" struct") {
            Some((true, auto_cleanup))
        } else if trimmed.ends_with(" union") {
            Some((false, auto_cleanup))
        } else {
            None
        }
    }
}
