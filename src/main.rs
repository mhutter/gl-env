#![forbid(unsafe_code)]

use std::io::{stdin, stdout};

use clap::Parser;
use gl_env::{
    cli::{Cli, Commands},
    gitlab::Gitlab,
    State,
};

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Diff(args) => {
            let gitlab = Gitlab::from(&args);
            diff(gitlab, &args.project);
        }
        Commands::Dump(args) => {
            let gitlab = Gitlab::from(&args);
            dump(gitlab, &args.project);
        }
        Commands::Update { args, dry_run } => {
            let gitlab = Gitlab::from(&args);
            update(gitlab, &args.project, dry_run);
        }
    }
}

/// Compare the desired to the actual state
fn diff(gitlab: Gitlab, project: &str) {
    // Re-encode YAML so we have a canonical representation
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired = serde_yml::to_string(&desired).unwrap();

    let actual: State = gitlab.list_project_variables(project).unwrap().into();
    let actual = serde_yml::to_string(&actual).unwrap();

    const GREEN: &str = "\x1b[0;32m";
    const RED: &str = "\x1b[0;31m";
    const RESET: &str = "\x1b[0m";

    for diff in diff::lines(&actual, &desired) {
        match diff {
            diff::Result::Left(l) => println!("{RED}-{l}{RESET}"),
            diff::Result::Both(l, _) => println!(" {l}"),
            diff::Result::Right(r) => println!("{GREEN}+{r}{RESET}"),
        }
    }
}

/// Fetch all currently configured variables, and dump them in YAML format to STDOUT
fn dump(gitlab: Gitlab, project: &str) {
    let vars = gitlab.list_project_variables(project).unwrap();
    let state = State::from(vars);
    serde_yml::to_writer(stdout(), &state).expect("Serialize data");
}

fn update(_gitlab: Gitlab, _project: &str, _dry_run: bool) {
    unimplemented!()
}
