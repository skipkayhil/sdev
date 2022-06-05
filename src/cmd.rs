use std::fmt;
use std::process::{Command, ExitStatus};

macro_rules! println_shell {
    ($($arg:tt)*) => ({
        println!("\x1b[90m$ {}\x1b[0m", format_args!($($arg)*));
    })
}

struct PrintableCommand {
    command: Command,
}

impl PrintableCommand {
    fn run(&mut self) -> ExitStatus {
        println_shell!("{}\n", self);

        self.command
            .status()
            .unwrap_or_else(|_| panic!("failed to execute {:?}", self.command.get_program()))
    }
}

impl fmt::Display for PrintableCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.command.get_program().to_str().unwrap())?;

        for arg in self.command.get_args() {
            write!(f, " {}", arg.to_str().unwrap())?;
        }

        Ok(())
    }
}

pub fn run_printable(command: Command) -> ExitStatus {
    PrintableCommand { command }.run()
}

mod env {
    pub fn home() -> String {
        std::env::var("HOME").expect("unknown HOME directory")
    }
}

pub mod git {
    use std::process::Command;

    use crate::config::Config;
    use crate::repo::Repo;

    pub fn clone_cmd(repo: &Repo, config: &Config) -> Command {
        let mut command = Command::new("git");

        command
            .arg("clone")
            .arg(repo.to_url())
            .arg(repo.to_absolute_path(&config.root));

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
