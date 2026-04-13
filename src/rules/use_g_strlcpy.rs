use tree_sitter::Node;

use super::{CheckContext, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGStrlcpy;

impl Rule for UseGStrlcpy {
    fn name(&self) -> &'static str {
        "use_g_strlcpy"
    }

    fn description(&self) -> &'static str {
        "Use g_strlcpy/g_strlcat instead of unsafe strcpy/strcat/strncat"
    }

    fn category(&self) -> super::Category {
        super::Category::Correctness
    }

    fn fixable(&self) -> bool {
        false
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

impl UseGStrlcpy {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        if node.kind() == "call_expression"
            && let Some(function) = node.child_by_field_name("function")
        {
            let func_name = ast_context.get_node_text(function, ctx.source);
            let message = match func_name {
                "strcpy" => Some(
                    "Use g_strlcpy(dst, src, sizeof(dst)) instead of strcpy — no bounds checking"
                        .to_string(),
                ),
                "strcat" => Some(
                    "Use g_strlcat(dst, src, sizeof(dst)) instead of strcat — no bounds checking"
                        .to_string(),
                ),
                "strncat" => Some(
                    "Use g_strlcat(dst, src, sizeof(dst)) instead of strncat — strncat's n parameter is the max to append, not the buffer size, which is error-prone"
                        .to_string(),
                ),
                _ => None,
            };

            if let Some(msg) = message {
                violations.push(self.violation(
                    ctx.file_path,
                    ctx.base_line + node.start_position().row,
                    node.start_position().column + 1,
                    msg,
                ));
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }
}
