use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGStrHasPrefixSuffix;

impl Rule for UseGStrHasPrefixSuffix {
    fn name(&self) -> &'static str {
        "use_g_str_has_prefix_suffix"
    }

    fn description(&self) -> &'static str {
        "Use g_str_has_prefix/g_str_has_suffix() instead of manual strncmp/strcmp comparisons"
    }

    fn category(&self) -> super::Category {
        super::Category::Style
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

impl UseGStrHasPrefixSuffix {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        if node.kind() == "binary_expression"
            && let Some(operator) = node.child_by_field_name("operator")
        {
            let op_text = ast_context.get_node_text(operator, ctx.source);
            if (op_text == "==" || op_text == "!=")
                && let Some(left) = node.child_by_field_name("left")
                && let Some(right) = node.child_by_field_name("right")
            {
                // strncmp(...) == 0  or  0 == strncmp(...)
                self.check_strncmp_prefix(ast_context, left, right, op_text, ctx, node, violations);
                self.check_strncmp_prefix(ast_context, right, left, op_text, ctx, node, violations);
                // strcmp(...) == 0  or  0 == strcmp(...)
                self.check_strcmp_suffix(ast_context, left, right, op_text, ctx, node, violations);
                self.check_strcmp_suffix(ast_context, right, left, op_text, ctx, node, violations);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_strncmp_prefix(
        &self,
        ast_context: &AstContext,
        strncmp_side: Node,
        value_side: Node,
        operator: &str,
        ctx: &CheckContext,
        parent_node: Node,
        violations: &mut Vec<Violation>,
    ) {
        if strncmp_side.kind() != "call_expression" {
            return;
        }

        let Some(function) = strncmp_side.child_by_field_name("function") else {
            return;
        };
        if ast_context.get_node_text(function, ctx.source) != "strncmp" {
            return;
        }

        if ast_context.get_node_text(value_side, ctx.source).trim() != "0" {
            return;
        }

        let Some(args) = strncmp_side.child_by_field_name("arguments") else {
            return;
        };

        let mut cursor = args.walk();
        let arguments: Vec<Node> = args
            .children(&mut cursor)
            .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
            .collect();

        if arguments.len() != 3 {
            return;
        }

        let str_arg = ast_context.get_node_text(arguments[0], ctx.source);
        let prefix_arg = arguments[1];
        let len_arg = arguments[2];

        if prefix_arg.kind() != "string_literal" {
            return;
        }
        let prefix_text = ast_context.get_node_text(prefix_arg, ctx.source);

        if !self.is_strlen_of(ast_context, len_arg, prefix_text, ctx.source) {
            return;
        }

        let spacing = ctx.source_text(function.end_byte(), args.start_byte());
        let replacement = if operator == "==" {
            format!("g_str_has_prefix{spacing}({str_arg}, {prefix_text})")
        } else {
            format!("!g_str_has_prefix{spacing}({str_arg}, {prefix_text})")
        };

        let fix = Fix::from_node(parent_node, ctx, &replacement);
        violations.push(self.violation_with_fix(
            ctx.file_path,
            ctx.base_line + parent_node.start_position().row,
            parent_node.start_position().column + 1,
            format!("Use {replacement} instead of strncmp() {operator} 0"),
            fix,
        ));
    }

    #[allow(clippy::too_many_arguments)]
    fn check_strcmp_suffix(
        &self,
        ast_context: &AstContext,
        strcmp_side: Node,
        value_side: Node,
        operator: &str,
        ctx: &CheckContext,
        parent_node: Node,
        violations: &mut Vec<Violation>,
    ) {
        if strcmp_side.kind() != "call_expression" {
            return;
        }

        let Some(function) = strcmp_side.child_by_field_name("function") else {
            return;
        };
        if ast_context.get_node_text(function, ctx.source) != "strcmp" {
            return;
        }

        if ast_context.get_node_text(value_side, ctx.source).trim() != "0" {
            return;
        }

        let Some(args) = strcmp_side.child_by_field_name("arguments") else {
            return;
        };

        let mut cursor = args.walk();
        let arguments: Vec<Node> = args
            .children(&mut cursor)
            .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
            .collect();

        if arguments.len() != 2 {
            return;
        }

        let offset_arg = arguments[0];
        let suffix_arg = arguments[1];

        if suffix_arg.kind() != "string_literal" {
            return;
        }
        let suffix_text = ast_context.get_node_text(suffix_arg, ctx.source);

        // First arg must be: <str_expr> + strlen(<str_expr>) - strlen("suffix")
        let Some(str_expr) =
            self.extract_suffix_base(ast_context, offset_arg, suffix_text, ctx.source)
        else {
            return;
        };

        let spacing = ctx.source_text(function.end_byte(), args.start_byte());
        let replacement = if operator == "==" {
            format!("g_str_has_suffix{spacing}({str_expr}, {suffix_text})")
        } else {
            format!("!g_str_has_suffix{spacing}({str_expr}, {suffix_text})")
        };

        let fix = Fix::from_node(parent_node, ctx, &replacement);
        violations.push(self.violation_with_fix(
            ctx.file_path,
            ctx.base_line + parent_node.start_position().row,
            parent_node.start_position().column + 1,
            format!("Use {replacement} instead of strcmp() {operator} 0"),
            fix,
        ));
    }

    /// Validates that `node` is `<str_expr> + strlen(<str_expr>) -
    /// strlen("suffix")` and returns `str_expr` if so.
    fn extract_suffix_base<'a>(
        &self,
        ast_context: &AstContext,
        node: Node,
        suffix_text: &str,
        source: &'a [u8],
    ) -> Option<&'a str> {
        // Top level: X - strlen("suffix")
        if node.kind() != "binary_expression" {
            return None;
        }
        let op = node.child_by_field_name("operator")?;
        if ast_context.get_node_text(op, source) != "-" {
            return None;
        }
        let lhs = node.child_by_field_name("left")?;
        let rhs = node.child_by_field_name("right")?;

        if !self.is_strlen_of(ast_context, rhs, suffix_text, source) {
            return None;
        }

        // Left side: <str_expr> + strlen(<str_expr>)
        if lhs.kind() != "binary_expression" {
            return None;
        }
        let inner_op = lhs.child_by_field_name("operator")?;
        if ast_context.get_node_text(inner_op, source) != "+" {
            return None;
        }
        let str_expr_node = lhs.child_by_field_name("left")?;
        let strlen_node = lhs.child_by_field_name("right")?;

        let str_expr = ast_context.get_node_text(str_expr_node, source);
        if !self.is_strlen_of(ast_context, strlen_node, str_expr, source) {
            return None;
        }

        Some(str_expr)
    }

    /// Returns true if `node` is `strlen(expected_text)`
    fn is_strlen_of(
        &self,
        ast_context: &AstContext,
        node: Node,
        expected_text: &str,
        source: &[u8],
    ) -> bool {
        if node.kind() != "call_expression" {
            return false;
        }
        let Some(func) = node.child_by_field_name("function") else {
            return false;
        };
        if ast_context.get_node_text(func, source) != "strlen" {
            return false;
        }
        let Some(args) = node.child_by_field_name("arguments") else {
            return false;
        };
        let mut cursor = args.walk();
        let inner_args: Vec<Node> = args
            .children(&mut cursor)
            .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
            .collect();
        if inner_args.len() != 1 {
            return false;
        }
        ast_context.get_node_text(inner_args[0], source) == expected_text
    }
}
