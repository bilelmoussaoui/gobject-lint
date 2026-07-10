use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::{Expression, SourceLocation};

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub location: SourceLocation,
}

impl Hash for SubscriptExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.array.hash(state);
        self.index.hash(state);
    }
}
