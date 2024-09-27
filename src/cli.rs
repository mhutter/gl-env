//! Definition of the command-line interfaces.
//!
//! Doc-comments in this file will likely show up in a help text.

use clap::{Args, Parser, Subcommand};
use url::Url;

use crate::gitlab::{Gitlab, Target};

/// Tools to bulk-edit Project-level CI/CD variables in GitLab.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path/ID of the project (either `mygroup/myproject` or numeric Project ID).
    #[arg(short, long, global = true)]
    pub project: Option<String>,

    /// Path/ID of the group (either `mygroup` or numeric Group ID).
    #[arg(short, long, global = true)]
    pub group: Option<String>,
}

impl From<&Cli> for Target {
    fn from(args: &Cli) -> Self {
        match (args.group.clone(), args.project.clone()) {
            (Some(v), None) => return Self::Group(v),
            (None, Some(v)) => return Self::Project(v),
            _ => panic!("either -g/--group or -p/--project must be passed"),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all configured CI/CD variables
    ///
    /// Output columns: Key, Environment, Masked, Protected, Raw
    List(CommonArgs),

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

        /// Remove variables not present in desired state
        #[arg(long)]
        prune: bool,
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
}

impl From<&CommonArgs> for Gitlab {
    fn from(args: &CommonArgs) -> Self {
        Self::new(&args.url, &args.token)
    }
}
