#![forbid(unsafe_code)]

use std::io::{stdin, stdout};

use clap::Parser;

use cli::{Cli, Commands};
use gitlab::{Gitlab, Target, Variable};
use state::State;

pub mod cli;
pub mod gitlab;
pub mod state;

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

const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GREY: &str = "\x1b[0;90m";
const RESET: &str = "\x1b[0m";

fn main() {
    let cli = Cli::parse();
    let target = Target::from(&cli);

    match cli.command {
        Commands::List(args) => {
            let gitlab = Gitlab::from(&args);
            list(&gitlab, target);
        }
        Commands::Diff(args) => {
            let gitlab = Gitlab::from(&args);
            diff(&gitlab, target);
        }
        Commands::Dump(args) => {
            let gitlab = Gitlab::from(&args);
            dump(&gitlab, target);
        }
        Commands::Apply {
            args,
            dry_run,
            prune,
        } => {
            let gitlab = Gitlab::from(&args);
            apply(&gitlab, target, prune, dry_run);
        }
    }
}

fn list(gitlab: &Gitlab, target: Target) {
    let mut variables = gitlab.list_variables(&target).unwrap();
    if variables.is_empty() {
        println!("{GREY}no variables defined{RESET}");
        return;
    }

    variables.sort();
    let key_len = variables.iter().map(|v| v.key.len()).max().unwrap().max(3);
    let env_len = variables
        .iter()
        .map(|v| v.environment_scope.len())
        .max()
        .unwrap()
        .max(3);

    let divider = format!(
        "+-{}-+-{}-+---+---+---+",
        "-".repeat(key_len),
        "-".repeat(env_len)
    );

    println!(
        "{}\n| {:key_len$} | {:env_len$} | m | p | r |\n{}",
        divider, "KEY", "ENV", divider
    );
    for v in variables {
        println!(
            "| {BLUE}{:key_len$}{RESET} | {:env_len$} | {GREEN}{}{RESET} | {GREEN}{}{RESET} | {GREEN}{}{RESET} |",
            v.key,
            v.environment_scope,
            if v.masked { 'x' } else { ' ' },
            if v.protected { 'x' } else { ' ' },
            if v.raw { 'x' } else { ' ' },
        );
    }
    println!("{divider}");
}

/// Apply all variables
fn apply(gitlab: &Gitlab, target: Target, prune: bool, dry_run: bool) {
    if dry_run {
        println!("Running in DRY RUN MODE");
    }
    // Load desired state
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired: Vec<Variable> = desired.into();

    // Load actual state
    let mut actual = gitlab.list_variables(&target).unwrap();
    // NOTE: This introduces a "time-of-check to time-of-use" situation; however at this point it
    // seems a bit too extreme to implement a locking mechanism (which would probably be relatively
    // easy to implement using a special variable).

    for variable in desired {
        // Check if the current variable already exists, and if so, remove it from the list.
        let existing = actual
            .iter()
            .position(|a| a.is_same(&variable))
            .map(|i| actual.remove(i));

        match existing {
            Some(v) if v == variable => println!("{variable} no change needed"),
            Some(_) => {
                if !dry_run {
                    gitlab.update_variable(&target, &variable).unwrap();
                }
                println!("{YELLOW}{variable} updated{RESET}");
            }
            None => {
                if !dry_run {
                    gitlab.create_variable(&target, &variable).unwrap();
                }
                println!("{GREEN}{variable} created{RESET}");
            }
        }
    }

    if prune {
        for variable in actual {
            gitlab.delete_variable(&target, &variable).unwrap();
            println!("{RED}{variable} deleted{RESET}");
        }
    } else {
        for variable in actual {
            println!("{RED}{variable} exists in GitLab, but not in desired state.{RESET}");
        }
    }
}

/// Compare the desired to the actual state
fn diff(gitlab: &Gitlab, target: Target) {
    // Re-encode YAML so we have a canonical representation
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired: Vec<Variable> = desired.into();
    let desired = serde_yml::to_string(&desired).unwrap();

    // Load actual variables, and translate to `State` and back to ensure both lists are sorted
    // identically
    let actual: State = gitlab.list_variables(&target).unwrap().into();
    let actual: Vec<Variable> = actual.into();
    let actual = serde_yml::to_string(&actual).unwrap();

    for diff in diff::lines(&actual, &desired) {
        match diff {
            diff::Result::Left(l) => println!("{RED}- {l}{RESET}"),
            diff::Result::Both(l, _) => println!("  {l}"),
            diff::Result::Right(r) => println!("{GREEN}+ {r}{RESET}"),
        }
    }
}

/// Fetch all currently configured variables, and dump them in YAML format to STDOUT
fn dump(gitlab: &Gitlab, target: Target) {
    let vars = gitlab.list_variables(&target).unwrap();
    let state = State::from(vars);
    serde_yml::to_writer(stdout(), &state).expect("Serialize data");
}
