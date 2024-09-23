#![forbid(unsafe_code)]

use clap::Parser;
use gl_env::{
    cli::{Cli, Commands},
    gitlab::Gitlab,
    State,
};

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Diff(_) => unimplemented!(),
        Commands::Dump(args) => {
            let gitlab = Gitlab::new(args.url, args.token);
            dump(gitlab, &args.project);
        }
        Commands::Update { .. } => unimplemented!(),
    }
}

/// Fetch all currently configured variables, and dump them in YAML format to STDOUT
fn dump(gitlab: Gitlab, project: &str) {
    let vars = gitlab.list_project_variables(project).unwrap();
    let state = State::from(vars);
    serde_yml::to_writer(std::io::stdout(), &state).expect("Serialize data");
}
