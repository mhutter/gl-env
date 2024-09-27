#![allow(clippy::result_large_err)]

use std::{io, time::Duration};

use serde::Serialize;
use url::Url;

use crate::APP_UA;

mod models;
pub use models::*;

pub type FetchResult<T> = std::result::Result<T, FetchError>;

/// Tiny GitLab SDK
///
/// Only implements the features we actually use
pub struct Gitlab {
    agent: ureq::Agent,
    url: Url,
    auth_header: String,
}

impl Gitlab {
    pub fn new(url: &Url, token: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .https_only(true)
            .redirects(0)
            .timeout(Duration::from_secs(10))
            .user_agent(APP_UA)
            .try_proxy_from_env(true)
            .build();

        let auth_header = format!("Bearer {token}");
        let url = url.join("api/v4/").unwrap();

        Self {
            agent,
            url,
            auth_header,
        }
    }

    /// List all CI/CD variables for a group or project
    pub fn list_variables(&self, target: &Target) -> FetchResult<Vec<Variable>> {
        let url = target.url_for_list(&self.url)?;
        self.get(&url)?.into_json().map_err(FetchError::from)
    }

    /// Create a new variable.
    ///
    /// If a variable with the same key already exists, the new variable must have a different
    /// `environment_scope`. Otherwise, GitLab returns a message similar to: `VARIABLE_NAME has
    /// already been taken`.
    pub fn create_variable(&self, target: &Target, variable: &Variable) -> FetchResult<Variable> {
        let url = target.url_for_list(&self.url)?;
        self.post(&url, variable)?
            .into_json()
            .map_err(FetchError::from)
    }

    /// Update a variable.
    pub fn update_variable(&self, target: &Target, variable: &Variable) -> FetchResult<Variable> {
        let url = target.url_for_item(&self.url, variable)?;
        dbg!(&url.as_str());
        self.put(&url, variable)?
            .into_json()
            .map_err(FetchError::from)
    }

    /// Delete a variable
    pub fn delete_variable(&self, target: &Target, variable: &Variable) -> FetchResult<()> {
        let url = target.url_for_item(&self.url, variable)?;
        self.delete(&url)?;
        Ok(())
    }

    /// Perform an authenticated GET request
    fn get(&self, url: &Url) -> FetchResult<ureq::Response> {
        self.agent
            .get(url.as_str())
            .set("Authorization", &self.auth_header)
            .call()
            .map_err(FetchError::from)
    }

    /// Perform an authenticated POST request
    fn post<T: Serialize>(&self, url: &Url, body: T) -> FetchResult<ureq::Response> {
        self.agent
            .post(url.as_str())
            .set("Authorization", &self.auth_header)
            .send_json(body)
            .map_err(FetchError::from)
    }

    /// Perform an authenticated PUT request
    fn put<T: Serialize>(&self, url: &Url, body: T) -> FetchResult<ureq::Response> {
        self.agent
            .put(url.as_str())
            .set("Authorization", &self.auth_header)
            .send_json(body)
            .map_err(FetchError::from)
    }

    /// Perform an authenticated DELETE request
    fn delete(&self, url: &Url) -> FetchResult<ureq::Response> {
        self.agent
            .delete(url.as_str())
            .set("Authorization", &self.auth_header)
            .call()
            .map_err(FetchError::from)
    }
}

/// Errors that might occur when performing requests to the GitLab API.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("Failed to send request: {0}")]
    RequestFailed(#[source] ureq::Transport),

    #[error("HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("Failed to deserialize Body: {0}")]
    Json(#[from] io::Error),
}

impl From<ureq::Error> for FetchError {
    fn from(value: ureq::Error) -> Self {
        match value {
            ureq::Error::Transport(transport) => Self::RequestFailed(transport),
            ureq::Error::Status(status, res) => {
                let body = res.into_string().unwrap_or_default();
                Self::HttpStatus { status, body }
            }
        }
    }
}

/// Either a Group or a Project
pub enum Target {
    Project(String),
    Group(String),
}

impl Target {
    pub fn url_for_list(&self, base: &Url) -> FetchResult<Url> {
        let (kind, target_id) = match self {
            Target::Project(p) => ("project", urlencode(p)),
            Target::Group(g) => ("group", urlencode(g)),
        };

        base.join(&format!("{kind}s/{target_id}/variables"))
            .map_err(FetchError::from)
    }

    pub fn url_for_item(&self, base: &Url, variable: &Variable) -> FetchResult<Url> {
        let key = variable.key.as_str();
        let mut url = self.url_for_list(base)?.join(&format!("variables/{key}"))?;
        url.query_pairs_mut()
            .append_pair("filter[environment_scope]", &variable.environment_scope);
        Ok(url)
    }
}

fn urlencode(s: &str) -> String {
    const RESERVED_CHARS: &[u8; 18] = b"!#$&'()*+,/:;=?@[]";
    let mut s = s.to_string();

    for c in RESERVED_CHARS {
        let t = format!("%{:X}", c);
        s = s.replace(*c as char, &t);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_encodes_str() {
        assert_eq!(urlencode("foo/bar"), "foo%2Fbar");
        assert_eq!(
            urlencode("!#$&'()*+,/:;=?@[]"),
            "%21%23%24%26%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D"
        );
    }
}
