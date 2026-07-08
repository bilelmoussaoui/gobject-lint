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
