use serde::{Deserialize, Serialize};

use crate::SourceLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub location: SourceLocation,
    pub fields: Vec<Field>,
    pub is_opaque: bool, // Only declared, not defined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_name: String,
}
