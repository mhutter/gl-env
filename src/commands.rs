use crate::{gitlab::Gitlab, State};

pub fn dump(gitlab: Gitlab, project: &str) {
    let vars = gitlab.list_project_variables(project).unwrap();
    let state = State {
        variables: vars.into(),
    };
    serde_yml::to_writer(std::io::stdout(), &state).expect("Serialize data");
}
