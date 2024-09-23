#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use gitlab::VariableType;

pub mod cli;
pub mod gitlab;

/// Name & version of the application
pub const APP: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Name, version & repository URL of the app
pub const APP_UA: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")",
);

/// Format of the YAML input/output
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub variables: Variables,
}

/// A map of variables, indexed by their key
#[derive(Debug, Serialize, Deserialize)]
pub struct Variables(BTreeMap<VariableKey, VariableValue>);

impl From<Vec<gitlab::Variable>> for Variables {
    fn from(value: Vec<gitlab::Variable>) -> Self {
        Self(
            value
                .into_iter()
                .map(|v| (VariableKey(v.key.clone()), VariableValue::from(v)))
                .collect(),
        )
    }
}

/// Key of a variable
///
/// Can only contain letters, numbers, and '_'.
///
/// TODO: validate this ^
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VariableKey(String);

/// The value of a variable.
#[derive(Debug, Serialize, Deserialize)]
pub struct VariableValue {
    /// The actual value
    ///
    /// If `masked` is `true`, this value must
    /// - be at least 8 characters long
    /// - not contain whitespace characters
    /// - not contain backslashes (`\`)
    ///
    /// TODO: validate this ^
    pub value: String,

    /// The description of the variable's value or usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Environment::is_default")]
    pub environment: Environment,

    /// Whether the value should be filtered in job logs.
    #[serde(skip_serializing_if = "is_false")]
    pub masked: bool,

    /// Export variable to pipelines running on protected branches and tags only.
    #[serde(skip_serializing_if = "is_false")]
    pub protected: bool,

    /// If `false`, `$` will be treated as the start of a reference to another variable.
    #[serde(skip_serializing_if = "is_false")]
    pub raw: bool,

    #[serde(rename = "type", skip_serializing_if = "VariableType::is_default")]
    pub variable_type: VariableType,
}

impl From<gitlab::Variable> for VariableValue {
    fn from(value: gitlab::Variable) -> Self {
        let gitlab::Variable {
            key: _,
            value,
            description,
            environment_scope,
            masked,
            protected,
            raw,
            variable_type,
        } = value;

        Self {
            value,
            description,
            environment: environment_scope.into(),
            masked,
            protected,
            raw,
            variable_type,
        }
    }
}

/// Environment scope of a variable
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Environment(String);

impl Environment {
    pub fn is_default(&self) -> bool {
        self.0 == "*"
    }
}

impl From<String> for Environment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self(String::from("*"))
    }
}

const fn is_false(value: &bool) -> bool {
    !(*value)
}
