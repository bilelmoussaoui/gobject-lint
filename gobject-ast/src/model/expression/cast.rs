use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::{Expression, SourceLocation, TypeInfo};

#[derive(Debug, Clone, Serialize)]
pub struct CastExpression {
    pub type_info: TypeInfo,
    pub operand: Box<Expression>,
    pub location: SourceLocation,
}

impl Hash for CastExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_info.hash(state);
        self.operand.hash(state);
    }
}
