use super::Violation;
use crate::ast_context::AstContext;
use crate::config::Config;

/// Rule that checks for function visibility issues using the AST model
/// This is much simpler than the tree-sitter version because we have structured data
pub struct FunctionVisibilityAst;

impl FunctionVisibilityAst {
    pub fn check_all(&self, ast_context: &AstContext, config: &Config) -> Vec<Violation> {
        if !config.rules.function_visibility {
            return vec![];
        }

        let mut violations = Vec::new();

        // Find all undeclared non-static functions
        for (path, func) in ast_context.find_undeclared_non_static_functions() {
            // Skip GObject internal functions
            if ast_context.is_gobject_internal(&func.name) {
                continue;
            }

            // Skip test functions in test directories
            if is_test_function(&func.name, path) {
                continue;
            }

            violations.push(Violation {
                file: path.display().to_string(),
                line: func.line,
                column: 1,
                message: format!(
                    "Function '{}' is not static and not declared in any header. Consider making it static or adding an export macro.",
                    func.name
                ),
                rule: "function_visibility".to_string(),
                snippet: None,
            });
        }

        violations
    }
}

/// Check if a function is a test function in a test directory
fn is_test_function(name: &str, path: &std::path::Path) -> bool {
    // Test functions typically start with "test_" and are in test directories
    if !name.starts_with("test_") {
        return false;
    }

    // Check if the path contains "test" or "tests" directory
    path.components().any(|component| {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(s) = os_str.to_str() {
                return s == "test" || s == "tests";
            }
        }
        false
    })
}
