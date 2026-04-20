use gobject_ast::{AssignmentOp, Expression, Statement, UnaryOp};

use super::{Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UseGSetObject;

impl Rule for UseGSetObject {
    fn name(&self) -> &'static str {
        "use_g_set_object"
    }

    fn description(&self) -> &'static str {
        "Suggest g_set_object() instead of manual g_clear_object and g_object_ref"
    }

    fn category(&self) -> super::Category {
        super::Category::Complexity
    }

    fn fixable(&self) -> bool {
        true
    }

    fn check_func_impl(
        &self,
        ast_context: &AstContext,
        _config: &Config,
        func: &gobject_ast::top_level::FunctionDefItem,
        path: &std::path::Path,
        violations: &mut Vec<Violation>,
    ) {
        let file = ast_context.project.files.get(path).unwrap();

        let start = func.location.start_byte;
        let end = func.location.end_byte;
        // Get the source for this function to preserve comments
        let func_source = &file.source[start..end];
        self.check_statements(&func.body_statements, path, func_source, start, violations);
    }
}

impl UseGSetObject {
    fn check_statements(
        &self,
        statements: &[Statement],
        file_path: &std::path::Path,
        source: &[u8],
        base_byte: usize,
        violations: &mut Vec<Violation>,
    ) {
        let mut i = 0;
        while i < statements.len() {
            // Check for g_clear_object(&var) followed by var = g_object_ref(...)
            if i + 1 < statements.len()
                && self.try_clear_then_ref(
                    &statements[i],
                    &statements[i + 1],
                    file_path,
                    source,
                    base_byte,
                    violations,
                )
            {
                i += 2;
                continue;
            }

            // Recurse into nested statements
            match &statements[i] {
                Statement::If(if_stmt) => {
                    self.check_statements(
                        &if_stmt.then_body,
                        file_path,
                        source,
                        base_byte,
                        violations,
                    );
                    if let Some(else_body) = &if_stmt.else_body {
                        self.check_statements(else_body, file_path, source, base_byte, violations);
                    }
                }
                Statement::Compound(compound) => {
                    self.check_statements(
                        &compound.statements,
                        file_path,
                        source,
                        base_byte,
                        violations,
                    );
                }
                Statement::Labeled(labeled) => {
                    self.check_statements(
                        std::slice::from_ref(&labeled.statement),
                        file_path,
                        source,
                        base_byte,
                        violations,
                    );
                }
                _ => {}
            }

            i += 1;
        }
    }

    /// Check for g_clear_object(&var)/g_object_unref(var) followed by var =
    /// g_object_ref(...)
    fn try_clear_then_ref(
        &self,
        s1: &Statement,
        s2: &Statement,
        file_path: &std::path::Path,
        source: &[u8],
        base_byte: usize,
        violations: &mut Vec<Violation>,
    ) -> bool {
        // First statement: g_clear_object(&var) or g_object_unref(var)
        let Some((var_name, needs_deref)) = self.extract_clear_or_unref_var(s1) else {
            return false;
        };

        // Second statement: var = g_object_ref(...) or *var = g_object_ref(...)
        let Some((assign_var, new_val)) = self.extract_object_ref_assignment(s2) else {
            return false;
        };

        // Check if variables match (accounting for * dereference)
        let expected_assign = if needs_deref {
            format!("*{}", var_name)
        } else {
            var_name.clone()
        };

        if assign_var != expected_assign {
            return false;
        }

        // g_set_object takes GObject**, so:
        // - If var is GObject* (needs_deref=false), use &var
        // - If var is GObject** (needs_deref=true), use var directly
        let set_object_call = if needs_deref {
            format!("g_set_object ({var_name}, {new_val});")
        } else {
            format!("g_set_object (&{var_name}, {new_val});")
        };

        // Extract bytes between the two statements to preserve comments
        let s1_end = s1.location().end_byte - base_byte;
        let s2_start = s2.location().start_byte - base_byte;
        let intermediate = std::str::from_utf8(&source[s1_end..s2_start]).unwrap_or("");
        let comment_prefix = intermediate.trim_start_matches(['\n', '\r', ' ', '\t']);

        // If there are comments, include them in the fix
        let fix_text = if comment_prefix.is_empty() {
            set_object_call.clone()
        } else {
            format!("{}{}", comment_prefix, set_object_call)
        };

        let fix = Fix::new(s1.location().start_byte, s2.location().end_byte, fix_text);

        violations.push(self.violation_with_fix(
            file_path,
            s1.location().line,
            s1.location().column,
            format!("Use {set_object_call} instead of g_clear_object and g_object_ref"),
            fix,
        ));
        true
    }

    /// Extract variable from g_clear_object(&var)/g_clear_object(ptr) or
    /// g_object_unref(var) Returns (var_name, needs_deref) where
    /// needs_deref indicates if assignment should use *var
    fn extract_clear_or_unref_var(&self, stmt: &Statement) -> Option<(String, bool)> {
        let Statement::Expression(expr_stmt) = stmt else {
            return None;
        };

        let Expression::Call(call) = &expr_stmt.expr else {
            return None;
        };

        if call.arguments.is_empty() {
            return None;
        }

        if call.is_function("g_clear_object") {
            // g_clear_object can take:
            // 1. &var - then assignment is var = ...
            // 2. ptr - then assignment is *ptr = ...
            let first_arg = call.get_arg(0)?;
            if let Expression::Unary(unary) = first_arg
                && unary.operator == UnaryOp::AddressOf
            {
                // Case 1: g_clear_object(&var)
                return Some((self.expr_to_string(&unary.operand), false));
            } else {
                // Case 2: g_clear_object(ptr) where ptr is GObject**
                return Some((self.expr_to_string(first_arg), true));
            }
        } else if call.is_function("g_object_unref") {
            // g_object_unref(var) - assignment is var = ...
            let first_arg = call.get_arg(0)?;
            return Some((self.expr_to_string(first_arg), false));
        }

        None
    }

    /// Extract (var, new_val) from var = g_object_ref(new_val)
    fn extract_object_ref_assignment(&self, stmt: &Statement) -> Option<(String, String)> {
        let Statement::Expression(expr_stmt) = stmt else {
            return None;
        };

        let Expression::Assignment(assign) = &expr_stmt.expr else {
            return None;
        };

        if assign.operator != AssignmentOp::Assign {
            return None;
        }

        // var = g_object_ref(new_val)
        if let Expression::Call(call) = &*assign.rhs
            && call.is_function("g_object_ref")
            && !call.arguments.is_empty()
        {
            let new_val = self.arg_to_string(&call.arguments[0]);
            let var_name = assign.lhs_as_text();
            if !var_name.is_empty() {
                return Some((var_name, new_val));
            }
        }

        None
    }

    fn arg_to_string(&self, arg: &gobject_ast::Argument) -> String {
        let gobject_ast::Argument::Expression(expr) = arg;
        self.expr_to_string(expr)
    }

    fn expr_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(id) => id.name.clone(),
            Expression::FieldAccess(f) => f.text(),
            Expression::Unary(unary) => {
                // Handle *ptr, &ptr, etc.
                format!(
                    "{}{}",
                    unary.operator.as_str(),
                    self.expr_to_string(&unary.operand)
                )
            }
            Expression::Call(call) => {
                // Reconstruct the call expression
                let args: Vec<String> = call
                    .arguments
                    .iter()
                    .map(|a| self.arg_to_string(a))
                    .collect();
                format!("{} ({})", call.function_name(), args.join(", "))
            }
            _ => String::new(),
        }
    }
}
