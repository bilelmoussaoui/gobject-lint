use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::{FieldAccessOp, SourceLocation, expression::Expression};

#[derive(Debug, Clone, Serialize)]
pub struct FieldAccessExpression {
    pub base: Box<Expression>,
    pub operator: FieldAccessOp,
    pub field: String,
    pub location: SourceLocation,
}

impl PartialEq for FieldAccessExpression {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.operator == other.operator && self.field == other.field
    }
}

impl Hash for FieldAccessExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.operator.hash(state);
        self.field.hash(state);
    }
}
