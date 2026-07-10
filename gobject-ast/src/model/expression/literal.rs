use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::model::SourceLocation;

#[derive(Debug, Clone, Serialize)]
pub struct StringLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

impl Hash for StringLiteralExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NumberLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

impl Hash for NumberLiteralExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CharLiteralExpression {
    pub value: String, // Like "'a'" or "'\\n'"
    pub location: SourceLocation,
}

impl Hash for CharLiteralExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NullExpression {
    pub location: SourceLocation,
}

impl Hash for NullExpression {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

#[derive(Debug, Clone, Serialize)]
pub struct BooleanExpression {
    pub value: bool,
    pub location: SourceLocation,
}

impl Hash for BooleanExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CommentExpression {
    pub text: String,
    pub location: SourceLocation,
}

impl Hash for CommentExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

/// Generic/unknown expression that we don't need to parse in detail
/// Used for offsetof, compound literals, etc. that don't affect linting rules
#[derive(Debug, Clone, Serialize)]
pub struct GenericExpression {
    pub text: String,
    pub location: SourceLocation,
}

impl Hash for GenericExpression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}
