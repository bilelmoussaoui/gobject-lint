use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGClearHandleId;

impl Rule for UseGClearHandleId {
    fn name(&self) -> &'static str {
        "use_g_clear_handle_id"
    }

    fn description(&self) -> &'static str {
        "Suggest g_clear_handle_id instead of manual cleanup and zero assignment"
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

impl UseGClearHandleId {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        // Track the byte range of any if-consequence we handle here so we can skip
        // it during recursion and avoid processing it a second time as a plain
        // compound_statement.
        let mut handled_consequence_range: Option<(usize, usize)> = None;

        // Check for if statements with g_clear_handle_id in braces that can be
        // simplified OR look for handle cleanup pattern to convert
        if node.kind() == "if_statement"
            && let Some(consequence) = node.child_by_field_name("consequence")
            && consequence.kind() == "compound_statement"
        {
            handled_consequence_range = Some((consequence.start_byte(), consequence.end_byte()));

            // First check if this is the pattern to convert (g_source_remove + id = 0)
            let conversions = self.check_cleanup_then_zero(ast_context, consequence, ctx.source);

            if !conversions.is_empty() {
                // This is a pattern to convert.
                // Count statements — if exactly 2, we can also remove the braces.
                let mut stmt_count = 0;
                let mut cursor = consequence.walk();
                for child in consequence.children(&mut cursor) {
                    if child.kind() == "expression_statement" {
                        stmt_count += 1;
                    }
                }

                // If the if has no else and the condition is just a zero-check on
                // the handle ID, the guard is redundant — g_clear_handle_id already
                // handles the id==0 case internally. Replace the whole if_statement.
                let has_else = node.child_by_field_name("alternative").is_some();
                let cond_id = node
                    .child_by_field_name("condition")
                    .and_then(|c| self.extract_id_from_condition(ast_context, c, ctx.source));

                for (var_name, cleanup_func, first_stmt, second_stmt) in conversions {
                    let position = first_stmt.start_position();
                    let replacement =
                        format!("g_clear_handle_id (&{}, {});", var_name, cleanup_func);

                    let can_remove_if =
                        !has_else && cond_id.is_some_and(|id| id == var_name) && stmt_count == 2;

                    let fix = if can_remove_if {
                        // The if guard is redundant — replace the whole if_statement
                        Fix::from_range(node.start_byte(), node.end_byte(), ctx, &replacement)
                    } else if stmt_count == 2 {
                        // Replace the entire compound_statement (including braces)
                        Fix::from_range(
                            consequence.start_byte(),
                            consequence.end_byte(),
                            ctx,
                            &replacement,
                        )
                    } else {
                        // Just replace the two statements
                        Fix::from_range(
                            first_stmt.start_byte(),
                            second_stmt.end_byte(),
                            ctx,
                            &replacement,
                        )
                    };

                    violations.push(self.violation_with_fix(
                        ctx.file_path,
                        ctx.base_line + position.row,
                        position.column + 1,
                        format!(
                            "Use {} instead of {} and zero assignment",
                            replacement, cleanup_func
                        ),
                        fix,
                    ));
                }
            } else {
                // Not a conversion pattern — check if braces around a single
                // g_clear_handle_id call can be removed.
                let mut stmt_count = 0;
                let mut clear_handle_call = None;
                let mut cursor = consequence.walk();
                for child in consequence.children(&mut cursor) {
                    if child.kind() == "expression_statement" {
                        stmt_count += 1;
                        if let Some(call) = ast_context.find_call_expression(child)
                            && let Some(function) = call.child_by_field_name("function")
                        {
                            let func_name = ast_context.get_node_text(function, ctx.source);
                            if func_name == "g_clear_handle_id" {
                                clear_handle_call = Some(child);
                            }
                        }
                    }
                }

                if stmt_count == 1
                    && let Some(call_stmt) = clear_handle_call
                {
                    let position = node.start_position();
                    let fix = Fix::from_range(
                        consequence.start_byte(),
                        consequence.end_byte(),
                        ctx,
                        ast_context.get_node_text(call_stmt, ctx.source),
                    );

                    violations.push(
                        self.violation_with_fix(
                            ctx.file_path,
                            ctx.base_line + position.row,
                            position.column + 1,
                            "Remove unnecessary braces around single g_clear_handle_id call"
                                .to_string(),
                            fix,
                        ),
                    );
                }
            }
        }

        // Look for cleanup+zero pairs directly in any compound_statement (e.g. a
        // function body or loop body), but skip compound_statements that were
        // already processed above as an if-consequence.
        if node.kind() == "compound_statement"
            && handled_consequence_range
                .map(|(s, e)| s != node.start_byte() || e != node.end_byte())
                .unwrap_or(true)
        {
            for (var_name, cleanup_func, first_stmt, second_stmt) in
                self.check_cleanup_then_zero(ast_context, node, ctx.source)
            {
                let position = first_stmt.start_position();
                let replacement = format!("g_clear_handle_id (&{}, {});", var_name, cleanup_func);

                let fix = Fix::from_range(
                    first_stmt.start_byte(),
                    second_stmt.end_byte(),
                    ctx,
                    &replacement,
                );

                violations.push(self.violation_with_fix(
                    ctx.file_path,
                    ctx.base_line + position.row,
                    position.column + 1,
                    format!(
                        "Use {} instead of {} and zero assignment",
                        replacement, cleanup_func
                    ),
                    fix,
                ));
            }
        }

        // Recurse, skipping the if-consequence we already handled above.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some((start, end)) = handled_consequence_range
                && child.start_byte() == start
                && child.end_byte() == end
            {
                continue;
            }
            self.check_node(ast_context, child, ctx, violations);
        }
    }

    /// Check for consecutive handle cleanup + var = 0
    /// Returns (var_name, cleanup_func, first_statement, second_statement)
    fn check_cleanup_then_zero<'a>(
        &self,
        ast_context: &AstContext,
        compound: Node<'a>,
        source: &'a [u8],
    ) -> Vec<(&'a str, &'a str, Node<'a>, Node<'a>)> {
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
                    && assign_var.trim() == var_name.trim()
                {
                    results.push((var_name, cleanup_func, first, second));
                }
            }
        }

        results
    }

    fn extract_handle_cleanup<'a>(
        &self,
        ast_context: &AstContext,
        node: Node,
        source: &'a [u8],
    ) -> Option<(&'a str, &'a str)> {
        if let Some(call) = ast_context.find_call_expression(node)
            && let Some(function) = call.child_by_field_name("function")
        {
            let func_name = ast_context.get_node_text(function, source);

            // Check if this is a known handle cleanup function
            let is_handle_cleanup = matches!(func_name, "g_source_remove" | "g_source_destroy");

            if !is_handle_cleanup {
                return None;
            }

            // Get the first argument (the handle ID variable)
            if let Some(args) = call.child_by_field_name("arguments") {
                let mut cursor = args.walk();
                for child in args.children(&mut cursor) {
                    if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
                        return Some((ast_context.get_node_text(child, source).trim(), func_name));
                    }
                }
            }
        }
        None
    }

    fn extract_zero_assignment<'a>(
        &self,
        ast_context: &AstContext,
        node: Node,
        source: &'a [u8],
    ) -> Option<&'a str> {
        if let Some(assignment) = self.find_assignment(node)
            && let Some(left) = assignment.child_by_field_name("left")
            && let Some(right) = assignment.child_by_field_name("right")
        {
            let right_text = ast_context.get_node_text(right, source);
            if right_text.trim() == "0" {
                return Some(ast_context.get_node_text(left, source).trim());
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

    /// Extract the handle ID from an if-condition, returning `Some(id_text)`
    /// when the condition is a redundant zero-check: `(id)`, `(id > 0)`,
    /// `(id != 0)`, `(0 < id)`, `(0 != id)`.
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
            // Plain truthy check: if (id)
            Some(ast_context.get_node_text(inner, source))
        }
    }
}
