use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::{Expression, SourceLocation};

#[derive(Debug, Clone, Serialize)]
pub struct ConditionalExpression {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
    pub location: SourceLocation,
}

impl Hash for ConditionalExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.condition.hash(state);
        self.then_expr.hash(state);
        self.else_expr.hash(state);
    }
}
