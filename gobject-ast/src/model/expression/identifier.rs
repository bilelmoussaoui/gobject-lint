use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::SourceLocation;

#[derive(Debug, Clone, Serialize)]
pub struct IdentifierExpression {
    pub name: String,
    pub location: SourceLocation,
}

impl PartialEq for IdentifierExpression {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for IdentifierExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
