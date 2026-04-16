use serde::{Deserialize, Serialize};

use crate::model::{
    expression::{Argument, CallExpression, Expression},
    statement::Statement,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub is_static: bool,
    pub export_macros: Vec<String>, // CLUTTER_EXPORT, G_MODULE_EXPORT, G_DEPRECATED_FOR, etc.
    pub has_static_forward_decl: bool, // Has a static forward declaration in the same file
    pub is_definition: bool,        // true = definition, false = declaration
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
    /// Byte range of the entire function (for definitions) - use with
    /// FileModel.source
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    /// Byte range of just the function body (for definitions) - use with
    /// FileModel.source
    pub body_start_byte: Option<usize>,
    pub body_end_byte: Option<usize>,
    /// Parsed body statements (for definitions) - ordered list
    pub body_statements: Vec<Statement>,
}

impl FunctionInfo {
    /// Find all calls to specific functions in the body
    /// Returns references to all CallExpression nodes that match any of the
    /// given function names
    pub fn find_calls<'a>(&'a self, function_names: &[&str]) -> Vec<&'a CallExpression> {
        self.find_calls_matching(|name| function_names.contains(&name))
    }

    /// Find all calls matching a predicate in the body
    /// Returns references to all CallExpression nodes where the predicate
    /// returns true
    pub fn find_calls_matching<F>(&self, predicate: F) -> Vec<&CallExpression>
    where
        F: Fn(&str) -> bool,
    {
        let mut calls = Vec::new();
        self.find_calls_recursive_matching(&self.body_statements, &predicate, &mut calls);
        calls
    }

    fn find_calls_recursive_matching<'a, F>(
        &'a self,
        statements: &'a [Statement],
        predicate: &F,
        calls: &mut Vec<&'a CallExpression>,
    ) where
        F: Fn(&str) -> bool,
    {
        for stmt in statements {
            match stmt {
                Statement::Expression(expr_stmt) => {
                    self.find_calls_in_expr_matching(&expr_stmt.expr, predicate, calls);
                }
                Statement::Return(ret) => {
                    if let Some(expr) = &ret.value {
                        self.find_calls_in_expr_matching(expr, predicate, calls);
                    }
                }
                Statement::Declaration(decl) => {
                    if let Some(expr) = &decl.initializer {
                        self.find_calls_in_expr_matching(expr, predicate, calls);
                    }
                }
                Statement::If(if_stmt) => {
                    self.find_calls_in_expr_matching(&if_stmt.condition, predicate, calls);
                    self.find_calls_recursive_matching(&if_stmt.then_body, predicate, calls);
                    if let Some(else_body) = &if_stmt.else_body {
                        self.find_calls_recursive_matching(else_body, predicate, calls);
                    }
                }
                Statement::Compound(compound) => {
                    self.find_calls_recursive_matching(&compound.statements, predicate, calls);
                }
                Statement::Labeled(labeled) => {
                    self.find_calls_recursive_matching(
                        std::slice::from_ref(&labeled.statement),
                        predicate,
                        calls,
                    );
                }
                _ => {}
            }
        }
    }

    fn find_calls_in_expr_matching<'a, F>(
        &'a self,
        expr: &'a Expression,
        predicate: &F,
        calls: &mut Vec<&'a CallExpression>,
    ) where
        F: Fn(&str) -> bool,
    {
        match expr {
            Expression::Call(call) => {
                if predicate(&call.function) {
                    calls.push(call);
                }
                // Also check arguments
                for arg in &call.arguments {
                    let Argument::Expression(e) = arg;
                    self.find_calls_in_expr_matching(e, predicate, calls);
                }
            }
            Expression::Assignment(assign) => {
                self.find_calls_in_expr_matching(&assign.rhs, predicate, calls);
            }
            Expression::Binary(binary) => {
                self.find_calls_in_expr_matching(&binary.left, predicate, calls);
                self.find_calls_in_expr_matching(&binary.right, predicate, calls);
            }
            Expression::Unary(unary) => {
                self.find_calls_in_expr_matching(&unary.operand, predicate, calls);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Option<String>,
    pub type_name: String,
}
