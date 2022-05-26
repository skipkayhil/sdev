mod env {
    pub fn home() -> String {
        std::env::var("HOME").expect("unknown HOME directory")
    }
}

pub mod git {
    use std::process::Command;

    use crate::cmd::env;
    use crate::repo::Repo;

    pub fn clone_cmd(repo: &Repo) -> Command {
        let mut command = Command::new("git");

        command
            .arg("clone")
            .arg(repo.to_url())
            .arg(repo.to_path_with_base(&env::home()));

        command
    }
}
