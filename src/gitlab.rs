#![allow(clippy::result_large_err)]

use std::{io, time::Duration};

use url::Url;

use crate::{cli::CommonArgs, APP_UA};

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
    pub fn new(url: Url, token: String) -> Self {
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

    /// List all available project variables
    ///
    /// TODO: include variables for non-default scopes
    pub fn list_project_variables(&self, project: &str) -> FetchResult<Vec<Variable>> {
        let project_id = project.replace("/", "%2F");
        let url = self
            .url
            .join(&format!("projects/{project_id}/variables"))
            .expect("projects URL");

        let res = self
            .agent
            .get(url.as_str())
            .set("Authorization", &self.auth_header)
            .call()?;

        let variables = res.into_json()?;
        Ok(variables)
    }
}

impl From<CommonArgs> for Gitlab {
    fn from(args: CommonArgs) -> Self {
        Self::new(args.url, args.token)
    }
}

/// Errors that might occur when performing requests to the GitLab API.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
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
