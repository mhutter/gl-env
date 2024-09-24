#![forbid(unsafe_code)]

use std::io::{stdin, stdout};

use clap::Parser;
use gl_env::{
    cli::{Cli, Commands},
    gitlab::{Gitlab, Variable},
    State, VariableValue,
};

const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const RESET: &str = "\x1b[0m";

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
        Commands::Apply { args, dry_run } => {
            let gitlab = Gitlab::from(&args);
            apply(gitlab, &args.project, dry_run);
        }
    }
}

/// Apply all variables
fn apply(gitlab: Gitlab, project: &str, dry_run: bool) {
    if dry_run {
        println!("Running in DRY RUN MODE");
    }
    // Load desired state
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");

    // Transpose to a `Vec<gitlab::Variable>`
    let mut variables = Vec::new();

    for (key, v) in desired.variables {
        let var = into_variable(key.into(), String::from("*"), v);
        variables.push(var);
    }

    for (env, vars) in desired.environments {
        for (key, v) in vars {
            let var = into_variable(key.into(), env.clone(), v);
            variables.push(var);
        }
    }

    // Load actual state
    let mut actual = gitlab.list_project_variables(project).unwrap();
    // NOTE: This introduces a "time-of-check to time-of-use" situation; however at this point it
    // seems a bit too extreme to implement a locking mechanism (which would probably be relatively
    // easy to implement using a special variable).

    for variable in variables {
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

    for variable in actual {
        println!("{RED}{variable} exists in GitLab, but not in desired state.{RESET}");
    }
}

fn into_variable(key: String, environment_scope: String, value: VariableValue) -> Variable {
    let VariableValue {
        value,
        description,
        masked,
        protected,
        raw,
        variable_type,
    } = value;
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

/// Compare the desired to the actual state
fn diff(gitlab: Gitlab, project: &str) {
    // Re-encode YAML so we have a canonical representation
    let desired: State = serde_yml::from_reader(stdin()).expect("Read desired state");
    let desired = serde_yml::to_string(&desired).unwrap();

    let actual: State = gitlab.list_project_variables(project).unwrap().into();
    let actual = serde_yml::to_string(&actual).unwrap();

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
