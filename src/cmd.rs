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

pub mod tmux {
    use std::process::Command;

    use crate::cmd::env;
    use crate::repo::Repo;

    pub fn attach_cmd(repo: &Repo) -> Command {
        let attach_command = match std::env::var("TMUX") {
            Ok(_) => "switch-client",
            Err(_) => "attach-session",
        };

        let mut command = Command::new("tmux");

        command.arg(attach_command).arg("-t").arg(repo.name());

        command
    }

    pub fn new_session(repo: &Repo) {
        Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(repo.name())
            .arg("-c")
            .arg(repo.to_path_with_base(&env::home()))
            .output()
            .expect("failed to execute 'tmux new-session'");
    }

    pub fn session_exists(repo: &Repo) -> bool {
        Command::new("tmux")
            .arg("has")
            .arg("-t")
            .arg(repo.name())
            .output()
            .expect("failed to execute 'tmux has'")
            .status
            .success()
    }
}
