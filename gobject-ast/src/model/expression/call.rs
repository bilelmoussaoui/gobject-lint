use serde::{Deserialize, Serialize};

use crate::model::{Expression, SourceLocation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub function: String,
    pub arguments: Vec<Argument>,
    pub location: SourceLocation,
}

impl CallExpression {
    /// Get argument as source text
    pub fn get_arg_text(&self, index: usize, source: &[u8]) -> Option<String> {
        self.arguments.get(index)?.to_source_string(source)
    }

    /// Check if this looks like a macro call (ALL_CAPS or ends with _)
    /// Examples: I_, N_, G_STRINGIFY, GINT_TO_POINTER
    pub fn is_likely_macro(&self) -> bool {
        self.function.chars().all(|c| c.is_uppercase() || c == '_') || self.function.ends_with('_')
    }

    /// Extract string literal from argument, unwrapping macro calls like
    /// I_("string") This is useful for g_param_spec calls where the name
    /// might be I_("property-name")
    pub fn extract_string_from_arg(&self, index: usize) -> Option<String> {
        let Argument::Expression(expr) = self.arguments.get(index)?;
        expr.extract_string_value()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    Expression(Box<Expression>),
    // Add more specific types as needed
}

impl Argument {
    /// Convert this argument back to source text
    pub fn to_source_string(&self, source: &[u8]) -> Option<String> {
        match self {
            Argument::Expression(expr) => expr.to_source_string(source),
        }
    }

    /// Check if this argument is a string literal or macro wrapping a string
    pub fn is_string_or_macro_string(&self) -> bool {
        let Argument::Expression(expr) = self;
        expr.is_string_or_macro_string()
    }

    /// Check if this argument is NULL
    pub fn is_null(&self) -> bool {
        let Argument::Expression(expr) = self;
        expr.is_null()
    }

    /// Extract string value from this argument, unwrapping macros
    pub fn extract_string_value(&self) -> Option<String> {
        let Argument::Expression(expr) = self;
        expr.extract_string_value()
    }
}
