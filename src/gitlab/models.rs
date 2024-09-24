use std::fmt;

use serde::{Deserialize, Serialize};

/// A Project's CI/CD variables
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub environment_scope: String,
    pub masked: bool,
    pub protected: bool,
    pub raw: bool,
    pub variable_type: VariableType,
}

impl Variable {
    /// Determines whether two variables are "the same one", meaning their `key` and
    /// `environment_scope` match.
    #[must_use]
    pub fn is_same(&self, other: &Self) -> bool {
        self.key == other.key && self.environment_scope == other.environment_scope
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.key, self.environment_scope)
    }
}

/// The type of a variable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    #[default]
    EnvVar,
    File,
}

impl VariableType {
    #[must_use]
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}
