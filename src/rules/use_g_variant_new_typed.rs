use gobject_ast::Expression;

use super::{Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGVariantNewTyped;

impl Rule for UseGVariantNewTyped {
    fn name(&self) -> &'static str {
        "use_g_variant_new_typed"
    }

    fn description(&self) -> &'static str {
        "Prefer g_variant_new_string/boolean/etc over g_variant_new with format strings"
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

                for call in func.find_calls(&["g_variant_new"]) {
                    self.check_call(path, call, &file.source, violations);
                }
            }
        }
    }
}

impl UseGVariantNewTyped {
    fn check_call(
        &self,
        file_path: &std::path::Path,
        call: &gobject_ast::CallExpression,
        source: &[u8],
        violations: &mut Vec<Violation>,
    ) {
        // Need at least 1 argument (the format string)
        if call.arguments.is_empty() {
            return;
        }

        // Check if first argument is a string literal
        let gobject_ast::Argument::Expression(first_expr) = &call.arguments[0];
        if !first_expr.is_string_literal() {
            return;
        }

        // Get the string literal value
        let Expression::StringLiteral(string_lit) = first_expr.as_ref() else {
            unreachable!();
        };

        let format_str = string_lit.value.trim_matches('"');

        // Map format string to typed function
        let typed_func = match format_str {
            "s" => "g_variant_new_string",
            "b" => "g_variant_new_boolean",
            "y" => "g_variant_new_byte",
            "n" => "g_variant_new_int16",
            "q" => "g_variant_new_uint16",
            "i" => "g_variant_new_int32",
            "u" => "g_variant_new_uint32",
            "x" => "g_variant_new_int64",
            "t" => "g_variant_new_uint64",
            "h" => "g_variant_new_handle",
            "d" => "g_variant_new_double",
            "o" => "g_variant_new_object_path",
            "g" => "g_variant_new_signature",
            "v" => "g_variant_new_variant",
            _ => return, // Not a simple type we can convert
        };

        // Collect remaining arguments (after format string)
        let rest_args = if call.arguments.len() > 1 {
            let rest: Vec<String> = call.arguments[1..]
                .iter()
                .filter_map(|arg| arg.to_source_string(source))
                .collect();
            rest.join(", ")
        } else {
            String::new()
        };

        // Build replacement
        let replacement = if rest_args.is_empty() {
            format!("{} ()", typed_func)
        } else {
            format!("{} ({})", typed_func, rest_args)
        };

        let fix = Fix::new(
            call.location.start_byte,
            call.location.end_byte,
            replacement.clone(),
        );

        violations.push(self.violation_with_fix(
            file_path,
            call.location.line,
            call.location.column,
            format!(
                "Use {} instead of g_variant_new(\"{}\", ...) for type safety",
                replacement, format_str
            ),
            fix,
        ));
    }
}
