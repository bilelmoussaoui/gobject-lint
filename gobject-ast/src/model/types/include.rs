use serde::{Deserialize, Serialize};

use crate::SourceLocation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub path: String,
    pub is_system: bool, // <> vs ""
    pub location: SourceLocation,
}
