#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use gitlab::{Variable, VariableType};

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

/// A map of variables, indexed by their key.
///
/// Must be a map such that is impossible to set the same variable twice.
pub type Variables = BTreeMap<VariableKey, VariableValue>;

/// Format of the YAML input/output
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    /// Default attributes to be applied to all variables
    #[serde(default)]
    pub defaults: Settings,

    /// Variables with no specific environment scope (`*`)
    #[serde(default)]
    pub variables: Variables,

    /// Variables belonging to a specific environment scope
    #[serde(default)]
    pub environments: BTreeMap<String, Variables>,
}

/// Variable settings
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Whether the value should be filtered in job logs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub masked: Option<bool>,

    /// Export variable to pipelines running on protected branches and tags only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,

    /// If `false`, `$` will be treated as the start of a reference to another variable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,
}

impl From<State> for Vec<Variable> {
    fn from(state: State) -> Self {
        let mut variables = Vec::new();

        for (key, v) in state.variables {
            let var = into_variable(key.into(), String::from("*"), v, &state.defaults);
            variables.push(var);
        }

        for (env, vars) in state.environments {
            for (key, v) in vars {
                let var = into_variable(key.into(), env.clone(), v, &state.defaults);
                variables.push(var);
            }
        }
        variables
    }
}

impl From<Vec<gitlab::Variable>> for State {
    fn from(variables: Vec<gitlab::Variable>) -> Self {
        let mut s = Self::default();
        for v in variables {
            let env = v.environment_scope.clone();
            let key = VariableKey(v.key.clone());
            let value = VariableValue::from(v);

            match env.as_str() {
                "*" => {
                    s.variables.insert(key, value);
                }
                _ => {
                    s.environments.entry(env).or_default().insert(key, value);
                }
            }
        }
        s
    }
}

fn into_variable(
    key: String,
    environment_scope: String,
    value: VariableValue,
    defaults: &Settings,
) -> Variable {
    let VariableValue {
        value,
        description,
        settings: Settings {
            masked,
            protected,
            raw,
        },
        variable_type,
    } = value;

    // Apply custom or GitLab defaults
    let masked = masked.unwrap_or(defaults.masked.unwrap_or(false));
    let protected = protected.unwrap_or(defaults.protected.unwrap_or(false));
    let raw = raw.unwrap_or(defaults.raw.unwrap_or(false));

    Variable {
        key,
        value,
        description,
        environment_scope,
        masked,
        protected,
        raw,
        variable_type,
    }
}

// impl AsRef<BTreeMap<VariableKey, VariableValue>

/// Key of a variable
///
/// Can only contain letters, numbers, and '_'.
///
/// TODO: validate this ^
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VariableKey(String);

impl From<VariableKey> for String {
    fn from(key: VariableKey) -> Self {
        key.0
    }
}

/// The value of a variable.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "VariableType::is_default"
    )]
    pub variable_type: VariableType,

    #[serde(flatten)]
    pub settings: Settings,
}

impl From<gitlab::Variable> for VariableValue {
    fn from(value: gitlab::Variable) -> Self {
        let gitlab::Variable {
            key: _,
            value,
            description,
            environment_scope: _,
            masked,
            protected,
            raw,
            variable_type,
        } = value;

        Self {
            value,
            description,
            settings: Settings {
                masked: Some(masked),
                protected: Some(protected),
                raw: Some(raw),
            },
            variable_type,
        }
    }
}
