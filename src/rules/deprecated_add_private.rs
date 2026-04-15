use super::Rule;
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct DeprecatedAddPrivate;

impl Rule for DeprecatedAddPrivate {
    fn name(&self) -> &'static str {
        "deprecated_add_private"
    }

    fn description(&self) -> &'static str {
        "Detect deprecated g_type_class_add_private (use G_DEFINE_TYPE_WITH_PRIVATE instead)"
    }

    fn category(&self) -> super::Category {
        super::Category::Restriction
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

                for call in func.find_calls(&["g_type_class_add_private"]) {
                    violations.push(self.violation(
                        path,
                        call.location.line,
                        call.location.column,
                        "g_type_class_add_private is deprecated since GLib 2.58. Use G_DEFINE_TYPE_WITH_PRIVATE or G_ADD_PRIVATE instead".to_string(),
                    ));
                }
            }
        }
    }
}
