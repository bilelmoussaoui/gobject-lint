use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::{AssignmentOp, Expression, SourceLocation};

#[derive(Debug, Clone, Serialize)]
pub struct Assignment {
    pub lhs: Box<Expression>, // Can be Identifier or FieldAccess
    pub operator: AssignmentOp,
    pub rhs: Box<Expression>,
    pub location: SourceLocation,
}

impl Hash for Assignment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lhs.hash(state);
        self.operator.hash(state);
        self.rhs.hash(state);
    }
}

impl Assignment {
    pub fn lhs_as_text(&self) -> &str {
        self.lhs.location().as_str().unwrap_or("")
    }
}
