use serde::{Deserialize, Serialize};

use crate::model::{Expression, SourceLocation, Statement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchStatement {
    pub condition: Expression,
    pub condition_location: SourceLocation,
    pub body: Vec<Statement>,
    pub location: SourceLocation,
}
