use std::collections::HashMap;

use gobject_ast::Statement;

use super::Rule;
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGAutolist;

impl Rule for UseGAutolist {
    fn name(&self) -> &'static str {
        "use_g_autolist"
    }

    fn description(&self) -> &'static str {
        "Suggest g_autolist/g_autoslist instead of manual g_list_free_full/g_slist_free_full cleanup"
    }

    fn category(&self) -> super::Category {
        super::Category::Complexity
    }

    fn check_func_impl(
        &self,
        _ast_context: &AstContext,
        _config: &Config,
        func: &gobject_ast::top_level::FunctionDefItem,
        path: &std::path::Path,
        violations: &mut Vec<Violation>,
    ) {
        self.check_function(func, path, violations);
    }
}

impl UseGAutolist {
    fn check_function(
        &self,
        func: &gobject_ast::top_level::FunctionDefItem,
        file_path: &std::path::Path,
        violations: &mut Vec<Violation>,
    ) {
        // Find all GList*/GSList* declarations
        let list_vars = self.find_list_vars(&func.body_statements);

        // For each list variable, check if it's freed with
        // g_list_free_full/g_slist_free_full
        for (type_info, location) in list_vars.values() {
            let free_func = if type_info.base_type == "GList" {
                "g_list_free_full"
            } else {
                "g_slist_free_full"
            };

            if func.is_var_passed_to_function(type_info, free_func, 0) {
                // Check if variable is returned (would need different handling)
                let is_returned = func.is_var_returned(type_info);

                if !is_returned {
                    let (auto_type, base_type) = match type_info.base_type.as_str() {
                        "GList" => ("g_autolist", "g_list"),
                        "GSList" => ("g_autoslist", "g_slist"),
                        _ => unreachable!(),
                    };

                    violations.push(self.violation(
                        file_path,
                        location.line,
                        location.column,
                        format!(
                            "Consider using {auto_type} to avoid manual {base_type}_free_full cleanup",
                        ),
                    ));
                }
            }
        }
    }

    /// Find all GList*/GSList* variable declarations
    fn find_list_vars(
        &self,
        statements: &[Statement],
    ) -> HashMap<String, (gobject_ast::TypeInfo, gobject_ast::SourceLocation)> {
        let mut result = HashMap::new();

        for stmt in statements {
            for decl in stmt.iter_declarations() {
                // Skip variables already using auto-cleanup macros
                if decl.type_info.uses_auto_cleanup() {
                    continue;
                }

                // Look for GList* or GSList*
                if (decl.type_info.base_type == "GList" || decl.type_info.base_type == "GSList")
                    && decl.type_info.is_pointer()
                    && decl.is_simple_identifier()
                {
                    result.insert(decl.name.clone(), (decl.type_info.clone(), decl.location));
                }
            }
        }

        result
    }
}
