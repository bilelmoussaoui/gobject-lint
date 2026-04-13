use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGAsciiFunctions;

/// Maps locale-dependent C ctype/string functions to their GLib ASCII-safe
/// equivalents
fn g_ascii_replacement(func_name: &str) -> Option<&'static str> {
    match func_name {
        "tolower" => Some("g_ascii_tolower"),
        "toupper" => Some("g_ascii_toupper"),
        "isdigit" => Some("g_ascii_isdigit"),
        "isalpha" => Some("g_ascii_isalpha"),
        "isalnum" => Some("g_ascii_isalnum"),
        "isspace" => Some("g_ascii_isspace"),
        "isupper" => Some("g_ascii_isupper"),
        "islower" => Some("g_ascii_islower"),
        "isxdigit" => Some("g_ascii_isxdigit"),
        "ispunct" => Some("g_ascii_ispunct"),
        "isprint" => Some("g_ascii_isprint"),
        "isgraph" => Some("g_ascii_isgraph"),
        "iscntrl" => Some("g_ascii_iscntrl"),
        _ => None,
    }
}

impl Rule for UseGAsciiFunctions {
    fn name(&self) -> &'static str {
        "use_g_ascii_functions"
    }

    fn description(&self) -> &'static str {
        "Use g_ascii_* functions instead of locale-dependent C ctype functions"
    }

    fn category(&self) -> super::Category {
        super::Category::Correctness
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

impl UseGAsciiFunctions {
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
            if let Some(replacement) = g_ascii_replacement(func_name) {
                let fix = Fix::from_node(function, ctx, replacement);
                violations.push(self.violation_with_fix(
                    ctx.file_path,
                    ctx.base_line + node.start_position().row,
                    node.start_position().column + 1,
                    format!(
                        "Use {replacement}() instead of {func_name}() — C ctype functions are locale-dependent"
                    ),
                    fix,
                ));
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }
}
