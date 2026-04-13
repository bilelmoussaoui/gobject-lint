use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGClearSignalHandler;

impl Rule for UseGClearSignalHandler {
    fn name(&self) -> &'static str {
        "use_g_clear_signal_handler"
    }

    fn description(&self) -> &'static str {
        "Use g_clear_signal_handler() instead of g_signal_handler_disconnect() and zeroing the ID"
    }

    fn category(&self) -> super::Category {
        super::Category::Complexity
    }

    fn fixable(&self) -> bool {
        true
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

                if let Some(func_source) = ast_context.get_function_source(path, func)
                    && let Some(tree) = ast_context.parse_c_source(func_source)
                {
                    let ctx = CheckContext {
                        source: func_source,
                        file_path: path,
                        base_line: func.line,
                        base_byte: func.start_byte.unwrap_or(0),
                    };
                    self.check_node(ast_context, tree.root_node(), &ctx, violations);
                }
            }
        }
    }
}

impl UseGClearSignalHandler {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        if node.kind() == "compound_statement" {
            self.check_compound(ast_context, node, ctx, violations);
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }

    fn check_compound(
        &self,
        ast_context: &AstContext,
        compound: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        let mut cursor = compound.walk();
        let stmts: Vec<Node> = compound
            .children(&mut cursor)
            .filter(|n| n.kind() != "{" && n.kind() != "}" && n.kind() != "comment")
            .collect();

        let mut i = 0;
        while i < stmts.len() {
            // if (id) / if (id > 0) { disconnect; id = 0; } — replace entire if_statement
            if self.try_if_guarded(ast_context, stmts[i], ctx, violations) {
                i += 1;
                continue;
            }

            // g_signal_handler_disconnect(obj, id); id = 0;
            if i + 1 < stmts.len()
                && self.try_disconnect_then_zero(
                    ast_context,
                    stmts[i],
                    stmts[i + 1],
                    ctx,
                    violations,
                )
            {
                i += 2;
                continue;
            }

            // bare g_signal_handler_disconnect(obj, struct->member) — no zero-assign
            if self.try_bare_disconnect_on_member(ast_context, stmts[i], &stmts, ctx, violations) {
                i += 1;
                continue;
            }

            self.check_node(ast_context, stmts[i], ctx, violations);
            i += 1;
        }
    }

    /// Matches `if (id) { g_signal_handler_disconnect(obj, id); id = 0; }`
    /// (also `if (id > 0)`, `if (id != 0)`) and replaces the entire
    /// if_statement — the guard is redundant since g_clear_signal_handler
    /// already skips the call when *id == 0.
    fn try_if_guarded(
        &self,
        ast_context: &AstContext,
        if_node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) -> bool {
        if if_node.kind() != "if_statement" {
            return false;
        }
        if if_node.child_by_field_name("alternative").is_some() {
            return false;
        }

        let Some(condition) = if_node.child_by_field_name("condition") else {
            return false;
        };
        let Some(guarded_id) = self.extract_id_from_condition(ast_context, condition, ctx.source)
        else {
            return false;
        };

        let Some(consequence) = if_node.child_by_field_name("consequence") else {
            return false;
        };
        if consequence.kind() != "compound_statement" {
            return false;
        }

        let mut cursor = consequence.walk();
        let stmts: Vec<Node> = consequence
            .children(&mut cursor)
            .filter(|n| n.kind() != "{" && n.kind() != "}" && n.kind() != "comment")
            .collect();

        if stmts.len() != 2 {
            return false;
        }

        let Some((obj, handler_id)) =
            self.extract_disconnect_args(ast_context, stmts[0], ctx.source)
        else {
            return false;
        };

        // The guarded ID must match the disconnect's handler_id arg
        if handler_id != guarded_id {
            return false;
        }

        if !self.is_zero_assign(ast_context, stmts[1], handler_id, ctx.source) {
            return false;
        }

        let replacement = format!("g_clear_signal_handler (&{handler_id}, {obj});");
        let fix = Fix::from_range(if_node.start_byte(), if_node.end_byte(), ctx, &replacement);
        violations.push(self.violation_with_fix(
            ctx.file_path,
            ctx.base_line + if_node.start_position().row,
            if_node.start_position().column + 1,
            format!("Use {replacement} instead of if-guarded g_signal_handler_disconnect"),
            fix,
        ));
        true
    }

    /// Matches a bare `g_signal_handler_disconnect(obj, struct->member)` call
    /// (no following zero-assign). When the handler ID is a struct member field
    /// (`->` access), the stored ID should be managed with
    /// g_clear_signal_handler so it is automatically zeroed after
    /// disconnect.
    ///
    /// Skipped when the base pointer of the handler ID is freed/unreffed within
    /// the same compound block — in that case the zero-assign is pointless.
    fn try_bare_disconnect_on_member(
        &self,
        ast_context: &AstContext,
        stmt: Node,
        all_stmts: &[Node],
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) -> bool {
        let Some((obj, handler_id)) = self.extract_disconnect_args(ast_context, stmt, ctx.source)
        else {
            return false;
        };

        // Only flag when the handler ID is a struct member access (contains ->)
        // Local-variable IDs (e.g. `handler_id`) are intentionally excluded.
        if !handler_id.contains("->") {
            return false;
        }

        // Extract the base pointer: `closure` from `closure->stopped_handler_id`.
        let base = handler_id.split("->").next().unwrap_or("").trim();
        if base.is_empty() {
            return false;
        }

        // Skip when the base struct (holder of the ID) is freed in the same block,
        // or when the signal source object (obj) is unreffed/cleared — in both
        // cases zeroing the stored ID is pointless since everything is going away.
        if self.is_freed_in_stmts(ast_context, all_stmts, base, ctx.source)
            || self.is_freed_in_stmts(ast_context, all_stmts, obj, ctx.source)
        {
            return false;
        }

        let replacement = format!("g_clear_signal_handler (&{handler_id}, {obj});");
        let fix = Fix::from_range(stmt.start_byte(), stmt.end_byte(), ctx, &replacement);
        violations.push(self.violation_with_fix(
            ctx.file_path,
            ctx.base_line + stmt.start_position().row,
            stmt.start_position().column + 1,
            format!("Use {replacement} instead of g_signal_handler_disconnect (also zeroes the stored ID)"),
            fix,
        ));
        true
    }

    /// Returns true if any statement in `stmts` is a call to a cleanup function
    /// (free/unref/destroy/clear_object/clear_pointer/…) that takes `target` as
    /// one of its arguments — either as `target` directly or as `&target`
    /// (for `g_clear_object (&x)` style calls).
    fn is_freed_in_stmts(
        &self,
        ast_context: &AstContext,
        stmts: &[Node],
        target: &str,
        source: &[u8],
    ) -> bool {
        for stmt in stmts {
            if stmt.kind() != "expression_statement" {
                continue;
            }
            let Some(call) = ast_context.find_call_expression(*stmt) else {
                continue;
            };
            let Some(function) = call.child_by_field_name("function") else {
                continue;
            };
            let func_name = ast_context.get_node_text(function, source);
            if !func_name.contains("free")
                && !func_name.contains("unref")
                && !func_name.contains("destroy")
                && !func_name.contains("clear")
            {
                continue;
            }
            let Some(args) = call.child_by_field_name("arguments") else {
                continue;
            };
            let mut cursor = args.walk();
            let freed = args
                .children(&mut cursor)
                .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
                .any(|arg| {
                    let arg_text = ast_context.get_node_text(arg, source);
                    // match both `target` and `&target` (g_clear_object style)
                    arg_text == target || arg_text.strip_prefix('&').is_some_and(|s| s == target)
                });
            if freed {
                return true;
            }
        }
        false
    }

    /// Matches `g_signal_handler_disconnect(obj, id); id = 0;`
    fn try_disconnect_then_zero(
        &self,
        ast_context: &AstContext,
        s1: Node,
        s2: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) -> bool {
        let Some((obj, handler_id)) = self.extract_disconnect_args(ast_context, s1, ctx.source)
        else {
            return false;
        };

        if !self.is_zero_assign(ast_context, s2, handler_id, ctx.source) {
            return false;
        }

        let replacement = format!("g_clear_signal_handler (&{handler_id}, {obj});");
        let fix = Fix::from_range(s1.start_byte(), s2.end_byte(), ctx, &replacement);
        violations.push(self.violation_with_fix(
            ctx.file_path,
            ctx.base_line + s1.start_position().row,
            s1.start_position().column + 1,
            format!("Use {replacement} instead of g_signal_handler_disconnect and zeroing the ID"),
            fix,
        ));
        true
    }

    /// Extract `(obj_text, handler_id_text)` from a
    /// `g_signal_handler_disconnect(obj, id)` call statement.
    fn extract_disconnect_args<'a>(
        &self,
        ast_context: &AstContext,
        stmt: Node,
        source: &'a [u8],
    ) -> Option<(&'a str, &'a str)> {
        if stmt.kind() != "expression_statement" {
            return None;
        }
        let call = ast_context.find_call_expression(stmt)?;
        let function = call.child_by_field_name("function")?;
        if ast_context.get_node_text(function, source) != "g_signal_handler_disconnect" {
            return None;
        }
        let args = call.child_by_field_name("arguments")?;
        let mut cursor = args.walk();
        let arg_nodes: Vec<Node> = args
            .children(&mut cursor)
            .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
            .collect();
        if arg_nodes.len() != 2 {
            return None;
        }
        Some((
            ast_context.get_node_text(arg_nodes[0], source),
            ast_context.get_node_text(arg_nodes[1], source),
        ))
    }

    /// Returns true if `stmt` is `expected_id = 0;`
    fn is_zero_assign(
        &self,
        ast_context: &AstContext,
        stmt: Node,
        expected_id: &str,
        source: &[u8],
    ) -> bool {
        if stmt.kind() != "expression_statement" {
            return false;
        }
        let Some(assignment) = self.find_assignment(stmt) else {
            return false;
        };
        let Some(left) = assignment.child_by_field_name("left") else {
            return false;
        };
        let Some(right) = assignment.child_by_field_name("right") else {
            return false;
        };
        ast_context.get_node_text(left, source) == expected_id
            && ast_context.get_node_text(right, source) == "0"
    }

    fn find_assignment<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .find(|&child| child.kind() == "assignment_expression")
    }

    /// Extract the handler ID expression from an if-condition.
    /// Handles `(id)`, `(id > 0)`, `(id != 0)`, `(0 < id)`.
    fn extract_id_from_condition<'a>(
        &self,
        ast_context: &AstContext,
        condition: Node,
        source: &'a [u8],
    ) -> Option<&'a str> {
        if condition.kind() != "parenthesized_expression" {
            return None;
        }
        let mut cursor = condition.walk();
        let inner = condition
            .children(&mut cursor)
            .find(|n| n.kind() != "(" && n.kind() != ")")?;

        if inner.kind() == "binary_expression" {
            let op = inner.child_by_field_name("operator")?;
            let left = inner.child_by_field_name("left")?;
            let right = inner.child_by_field_name("right")?;
            let op_text = ast_context.get_node_text(op, source);
            let left_text = ast_context.get_node_text(left, source);
            let right_text = ast_context.get_node_text(right, source);
            match op_text {
                "!=" | ">" => {
                    if right_text == "0" {
                        Some(left_text)
                    } else if left_text == "0" {
                        Some(right_text)
                    } else {
                        None
                    }
                }
                "<" => {
                    if left_text == "0" {
                        Some(right_text)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            Some(ast_context.get_node_text(inner, source))
        }
    }
}
