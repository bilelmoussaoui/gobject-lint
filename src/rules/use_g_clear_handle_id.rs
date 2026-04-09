use tree_sitter::Node;

use super::Rule;
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGClearHandleId;

impl Rule for UseGClearHandleId {
    fn name(&self) -> &'static str {
        "use_g_clear_handle_id"
    }

    fn description(&self) -> &'static str {
        "Suggest g_clear_handle_id instead of manual cleanup and zero assignment"
    }

    fn check_all(
        &self,
        ast_context: &AstContext,
        _config: &Config,
        violations: &mut Vec<Violation>,
    ) {
        for (path, file) in ast_context.iter_c_files() {
            for func in &file.functions {
                if !func.is_definition {
                    continue;
                }

                if let Some(func_source) = ast_context.get_function_source(path, func) {
                    if let Some(tree) = ast_context.parse_c_source(func_source) {
                        self.check_node(
                            ast_context,
                            tree.root_node(),
                            func_source,
                            path,
                            func.line,
                            violations,
                        );
                    }
                }
            }
        }
    }
}

impl UseGClearHandleId {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        source: &[u8],
        file_path: &std::path::Path,
        base_line: usize,
        violations: &mut Vec<Violation>,
    ) {
        // Look for compound statements that might have handle cleanup followed by zero
        // assignment
        if node.kind() == "compound_statement" || node.kind() == "if_statement" {
            let body = if node.kind() == "if_statement" {
                node.child_by_field_name("consequence")
            } else {
                Some(node)
            };

            if let Some(body_node) = body {
                for (var_name, cleanup_func, cleanup_node) in
                    self.check_cleanup_then_zero(ast_context, body_node, source)
                {
                    let position = cleanup_node.start_position();
                    violations.push(self.violation(
                        file_path,
                        base_line + position.row,
                        position.column + 1,
                        format!(
                            "Use g_clear_handle_id(&{}, {}) instead of {} and zero assignment",
                            var_name, cleanup_func, cleanup_func
                        ),
                    ));
                }
            }
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, source, file_path, base_line, violations);
        }
    }

    /// Check for consecutive handle cleanup + var = 0
    fn check_cleanup_then_zero<'a>(
        &self,
        ast_context: &AstContext,
        compound: Node<'a>,
        source: &[u8],
    ) -> Vec<(String, String, Node<'a>)> {
        let mut cursor = compound.walk();
        let statements: Vec<_> = compound
            .children(&mut cursor)
            .filter(|n| n.kind() == "expression_statement")
            .collect();

        let mut results = Vec::new();

        // Look for consecutive pairs
        for i in 0..statements.len().saturating_sub(1) {
            let first = statements[i];
            let second = statements[i + 1];

            // Check if first is a handle cleanup function call
            if let Some((var_name, cleanup_func)) =
                self.extract_handle_cleanup(ast_context, first, source)
            {
                // Check if second is assignment to 0
                if let Some(assign_var) = self.extract_zero_assignment(ast_context, second, source)
                {
                    if assign_var.trim() == var_name.trim() {
                        results.push((var_name, cleanup_func, first));
                    }
                }
            }
        }

        results
    }

    fn extract_handle_cleanup(
        &self,
        ast_context: &AstContext,
        node: Node,
        source: &[u8],
    ) -> Option<(String, String)> {
        if let Some(call) = ast_context.find_call_expression(node) {
            if let Some(function) = call.child_by_field_name("function") {
                let func_name = ast_context.get_node_text(function, source);

                // Check if this is a known handle cleanup function
                let is_handle_cleanup = matches!(
                    func_name.as_str(),
                    "g_source_remove"
                        | "g_source_destroy"
                        | "g_signal_handler_disconnect"
                        | "g_signal_handler_block"
                        | "g_signal_handler_unblock"
                );

                if !is_handle_cleanup {
                    return None;
                }

                // Get the first argument (the handle ID variable)
                if let Some(args) = call.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    for child in args.children(&mut cursor) {
                        if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
                            return Some((
                                ast_context.get_node_text(child, source).trim().to_string(),
                                func_name,
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_zero_assignment(
        &self,
        ast_context: &AstContext,
        node: Node,
        source: &[u8],
    ) -> Option<String> {
        if let Some(assignment) = self.find_assignment(node) {
            if let Some(left) = assignment.child_by_field_name("left") {
                if let Some(right) = assignment.child_by_field_name("right") {
                    let right_text = ast_context.get_node_text(right, source);
                    if right_text.trim() == "0" {
                        return Some(ast_context.get_node_text(left, source).trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn find_assignment<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "assignment_expression" {
                return Some(child);
            }
            if let Some(assignment) = self.find_assignment(child) {
                return Some(assignment);
            }
        }
        None
    }
}
