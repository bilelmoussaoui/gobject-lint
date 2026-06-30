use gobject_ast::model::{
    CallExpression, Expression, FileModel, FunctionDefItem, SizeofOperand, TypeInfo, UnaryOp,
};

use crate::{
    ast_context::AstContext,
    config::Config,
    rules::{Fix, Rule, Violation},
};

pub struct UseGNew;

impl Rule for UseGNew {
    fn name(&self) -> &'static str {
        "use_g_new"
    }

    fn description(&self) -> &'static str {
        "Suggest g_new/g_new0 instead of g_malloc/g_malloc0 with sizeof for type safety"
    }

    fn category(&self) -> crate::rules::Category {
        crate::rules::Category::Complexity
    }

    fn fixable(&self) -> bool {
        true
    }

    fn check_func_impl(
        &self,
        _ast_context: &AstContext,
        config: &Config,
        func: &FunctionDefItem,
        file: &FileModel,
        violations: &mut Vec<Violation>,
    ) {
        for call in func.find_calls(&["g_malloc", "g_malloc0"]) {
            self.check_call(file, call, func, config, violations);
        }
    }
}

impl UseGNew {
    fn check_call(
        &self,
        file: &FileModel,
        call: &CallExpression,
        func: &FunctionDefItem,
        config: &Config,
        violations: &mut Vec<Violation>,
    ) {
        // Need exactly 1 argument
        if call.arguments.len() != 1 {
            return;
        }

        // Check if argument is sizeof(Type)
        let Some(arg_expr) = call.get_arg(0) else {
            return;
        };
        let Expression::Sizeof(sizeof_expr) = arg_expr else {
            return;
        };

        let resolved_type;
        let type_name = match &sizeof_expr.operand {
            Some(SizeofOperand::Type(t)) => {
                resolved_type = Self::full_type_name(t);
                resolved_type.as_str()
            }
            Some(SizeofOperand::Expression(expr)) => {
                if let Expression::Identifier(id) = expr.as_ref() {
                    let var_types = func.local_var_types();
                    if let Some(type_info) = var_types.get(id.name.as_str()) {
                        if type_info.pointer_depth > 0 {
                            return;
                        }
                        resolved_type = Self::full_type_name(type_info);
                        resolved_type.as_str()
                    } else if id.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                        id.name.as_str()
                    } else {
                        return;
                    }
                } else if let Expression::Unary(unary) = expr.as_ref()
                    && unary.operator == UnaryOp::Dereference
                    && let Expression::Identifier(id) = unary.operand.as_ref()
                {
                    let var_types = func.local_var_types();
                    if let Some(type_info) = var_types.get(id.name.as_str()) {
                        if type_info.pointer_depth < 1 {
                            return;
                        }
                        resolved_type = Self::full_type_name(type_info);
                        resolved_type.as_str()
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            }
            None => return,
        };

        let func_name = call.function_name();
        let suggested_func = if call.is_function("g_malloc0") {
            "g_new0"
        } else {
            "g_new"
        };

        let replacement = config.style.format_call(suggested_func, &[type_name, "1"]);
        let message = format!(
            "Use {} instead of {}(sizeof({})) for type safety",
            replacement, func_name, type_name
        );
        let fix = Fix::new(
            call.location.start_byte,
            call.location.end_byte,
            replacement,
        );

        violations.push(self.violation_with_fix_at(&file.path, &call.location, message, fix));
    }

    fn full_type_name(type_info: &TypeInfo) -> String {
        if type_info.is_struct {
            format!("struct {}", type_info.base_type)
        } else if type_info.is_union {
            format!("union {}", type_info.base_type)
        } else {
            type_info.base_type.clone()
        }
    }
}
