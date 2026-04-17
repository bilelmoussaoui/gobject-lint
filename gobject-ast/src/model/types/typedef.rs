use serde::{Deserialize, Serialize};

use crate::SourceLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedefInfo {
    pub name: String,
    pub location: SourceLocation,
    pub target_type: String,
}
