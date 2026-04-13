use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGStrHasPrefix;

impl Rule for UseGStrHasPrefix {
    fn name(&self) -> &'static str {
        "use_g_str_has_prefix"
    }

    fn description(&self) -> &'static str {
        "Use g_str_has_prefix() instead of strncmp(s, \"prefix\", strlen(\"prefix\")) == 0"
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

impl UseGStrHasPrefix {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        // Look for: strncmp(s, "literal", strlen("literal")) == 0
        if node.kind() == "binary_expression"
            && let Some(operator) = node.child_by_field_name("operator")
        {
            let op_text = ast_context.get_node_text(operator, ctx.source);
            if (op_text == "==" || op_text == "!=")
                && let Some(left) = node.child_by_field_name("left")
                && let Some(right) = node.child_by_field_name("right")
            {
                // strncmp(...) == 0  or  0 == strncmp(...)
                self.check_strncmp_comparison(
                    ast_context,
                    left,
                    right,
                    op_text,
                    ctx,
                    node,
                    violations,
                );
                self.check_strncmp_comparison(
                    ast_context,
                    right,
                    left,
                    op_text,
                    ctx,
                    node,
                    violations,
                );
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_strncmp_comparison(
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

        let value_text = ast_context.get_node_text(value_side, ctx.source).trim();
        if value_text != "0" {
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

        // prefix must be a string literal
        if prefix_arg.kind() != "string_literal" {
            return;
        }
        let prefix_text = ast_context.get_node_text(prefix_arg, ctx.source);

        // third arg must be strlen("same_literal")
        if !self.is_strlen_of(ast_context, len_arg, prefix_text, ctx.source) {
            return;
        }

        let spacing_start = function.end_byte();
        let spacing_end = args.start_byte();
        let spacing = ctx.source_text(spacing_start, spacing_end);

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

    /// Returns true if `node` is a call `strlen("literal")` where the argument
    /// matches `expected_literal`
    fn is_strlen_of(
        &self,
        ast_context: &AstContext,
        node: Node,
        expected_literal: &str,
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
        ast_context.get_node_text(inner_args[0], source) == expected_literal
    }
}
