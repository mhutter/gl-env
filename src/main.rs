#![forbid(unsafe_code)]

use std::io::{stdin, stdout};

use clap::Parser;
use gl_env::{
    cli::{Cli, Commands},
    gitlab::{Gitlab, Variable},
    State,
};

const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const GREY: &str = "\x1b[0;90m";
const RESET: &str = "\x1b[0m";

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::List(args) => {
            let gitlab = Gitlab::from(&args);
            list(&gitlab, &args.project);
        }
        Commands::Diff(args) => {
            let gitlab = Gitlab::from(&args);
            diff(&gitlab, &args.project);
        }
        Commands::Dump(args) => {
            let gitlab = Gitlab::from(&args);
            dump(&gitlab, &args.project);
        }
        Commands::Apply {
            args,
            dry_run,
            prune,
        } => {
            let gitlab = Gitlab::from(&args);
            apply(&gitlab, &args.project, prune, dry_run);
        }
    }
}

fn list(gitlab: &Gitlab, project: &str) {
    let mut variables = gitlab.list_project_variables(project).unwrap();
    if variables.len() < 1 {
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
            v.masked.then(|| 'x').unwrap_or_else(|| ' '),
            v.protected.then(|| 'x').unwrap_or_else(|| ' '),
            v.raw.then(|| 'x').unwrap_or_else(|| ' '),
        );
    }
    println!("{divider}");
}

/// Apply all variables
fn apply(gitlab: &Gitlab, project: &str, prune: bool, dry_run: bool) {
    if dry_run {
        println!("Running in DRY RUN MODE");
    }
    // Load desired state
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired: Vec<Variable> = desired.into();

    // Load actual state
    let mut actual = gitlab.list_project_variables(project).unwrap();
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
                    gitlab.update_project_variable(project, &variable).unwrap();
                }
                println!("{YELLOW}{variable} updated{RESET}");
            }
            None => {
                if !dry_run {
                    gitlab.create_project_variable(project, &variable).unwrap();
                }
                println!("{GREEN}{variable} created{RESET}");
            }
        }
    }

    if prune {
        for variable in actual {
            gitlab.delete_project_variable(project, &variable).unwrap();
            println!("{RED}{variable} deleted{RESET}");
        }
    } else {
        for variable in actual {
            println!("{RED}{variable} exists in GitLab, but not in desired state.{RESET}");
        }
    }
}

/// Compare the desired to the actual state
fn diff(gitlab: &Gitlab, project: &str) {
    // Re-encode YAML so we have a canonical representation
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired: Vec<Variable> = desired.into();
    let desired = serde_yml::to_string(&desired).unwrap();

    // Load actual variables, and translate to `State` and back to ensure both lists are sorted
    // identically
    let actual: State = gitlab.list_project_variables(project).unwrap().into();
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
fn dump(gitlab: &Gitlab, project: &str) {
    let vars = gitlab.list_project_variables(project).unwrap();
    let state = State::from(vars);
    serde_yml::to_writer(stdout(), &state).expect("Serialize data");
}
