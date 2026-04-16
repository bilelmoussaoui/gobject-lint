use serde::{Deserialize, Serialize};

use crate::model::SourceLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharLiteralExpression {
    pub value: String, // Like "'a'" or "'\\n'"
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullExpression {
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanExpression {
    pub value: bool,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentExpression {
    pub text: String,
    pub location: SourceLocation,
}

/// Generic/unknown expression that we don't need to parse in detail
/// Used for offsetof, compound literals, etc. that don't affect linting rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericExpression {
    pub text: String,
    pub location: SourceLocation,
}
