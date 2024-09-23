use serde::{Deserialize, Serialize};

/// A Project's CI/CD variables
#[derive(Debug, Serialize, Deserialize)]
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

/// The type of a variable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    #[default]
    EnvVar,
    File,
}

impl VariableType {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}
