//! Definition of the command-line interfaces.
//!
//! Doc-comments in this file will likely show up in a help text.

use clap::{Args, Parser, Subcommand};
use url::Url;

/// Tools to bulk-edit Project-level CI/CD variables in GitLab.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show a diff between actual and desired variables
    Diff(CommonArgs),

    /// Dump the current variables
    Dump(CommonArgs),

    /// Apply desired variables
    Apply {
        #[clap(flatten)]
        args: CommonArgs,

        /// Print what WOULD been done, but don't actually do it
        #[arg(short, long)]
        dry_run: bool,
    },
}

/// Arguments shared between all commands
#[derive(Args)]
pub struct CommonArgs {
    /// Base URL of the GitLab instance.
    #[arg(
        short,
        long,
        default_value = "https://gitlab.com",
        global = true,
        env = "GITLAB_URL"
    )]
    pub url: Url,

    /// Personal Access Token.
    ///
    /// Required scopes: `api`.
    #[arg(short, long, env = "GITLAB_TOKEN", hide_env_values = true)]
    pub token: String,

    /// Path/ID of the project (either `mygroup/myproject` or numeric Project ID).
    pub project: String,
}
