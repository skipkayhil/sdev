use std::fmt;
use std::process::Command;

use crate::repo::MaybeOwnedRepo;
use crate::Config;

macro_rules! println_shell {
    ($($arg:tt)*) => ({
        println!("\x1b[90m$ {}\x1b[0m", format_args!($($arg)*));
    })
}

macro_rules! shell {
    ($bin:expr, $($x:expr),* $(,)?) => {
        {
            let mut cmd = Command::new($bin);
            $(cmd.arg($x);)*
            cmd
        }
    };
}

struct PrintableCommand {
    command: Command,
}

impl PrintableCommand {
    fn run(&mut self) -> Result<(), String> {
        println_shell!("{}\n", self);

        match self.command.status() {
            Ok(_) => Ok(()),
            Err(_) => Err(self.error_message()),
        }
    }

    fn error_message(&self) -> String {
        format!("failed to execute `{}`", &self)
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

pub fn run_printable(command: Command) -> Result<(), String> {
    PrintableCommand { command }.run()
}

pub fn clone(repo_arg: &MaybeOwnedRepo, config: Config) -> Result<(), String> {
    let owner = repo_arg.owner().as_ref().unwrap_or(&config.user);

    let command = shell!(
        "git",
        "clone",
        format!("git@github.com:{}/{}.git", owner, repo_arg.name()),
        config.root.join(owner).join(repo_arg.name()),
    );

    run_printable(command)
}

pub fn tmux(repo_arg: &MaybeOwnedRepo, config: Config) -> Result<(), String> {
    let mut command = shell!("tmux", "has", "-t", format!("={}", repo_arg.name()));

    match command.output() {
        Err(_) => return Err(PrintableCommand { command }.error_message()),
        Ok(output) if !output.status.success() => tmux::create_session(repo_arg, &config)?,
        _ => {}
    }

    run_printable(tmux::attach_cmd(repo_arg.name()))
}

pub mod find {
    use std::fs::DirEntry;
    use std::path::PathBuf;

    use crate::config::Config;

    pub fn repo(name: &str, config: &Config) -> Result<PathBuf, String> {
        let owners_path = config.root.join(&config.user).join(name);

        if owners_path.is_dir() {
            return Ok(owners_path);
        }

        let owners: Vec<DirEntry> = config
            .root
            .read_dir()
            .unwrap()
            .filter_map(|r| r.ok())
            .filter(|dir_entry| dir_entry.path().join(name).is_dir())
            .collect();

        match &owners[..] {
            [owner] => Ok(owner.path().join(name)),
            [] => Err(format!("No repos named \"{}\" found", name)),
            _ => Err(format!(
                "Multiple owners found for \"{}\" repo: {:?}",
                name,
                owners.iter().map(|o| o.file_name()).collect::<Vec<_>>()
            )),
        }
    }
}

pub mod tmux {
    use std::process::Command;

    use crate::cmd::{find, PrintableCommand};
    use crate::config::Config;
    use crate::repo::MaybeOwnedRepo;

    pub fn create_session(repo_arg: &MaybeOwnedRepo, config: &Config) -> Result<(), String> {
        let path = match repo_arg.owner() {
            Some(owner) => {
                let path = config.root.join(owner).join(repo_arg.name());

                if !path.is_dir() {
                    return Err(format!(
                        "{} does not exist. Has it been cloned?",
                        path.display()
                    ));
                }

                path
            }
            None => find::repo(repo_arg.name(), config)?,
        };

        let mut command = shell!(
            "tmux",
            "new-session",
            "-d",
            "-s",
            repo_arg.name(),
            "-c",
            path,
        );

        match command.output() {
            Err(_) => Err(PrintableCommand { command }.error_message()),
            _ => Ok(()),
        }
    }

    pub fn attach_cmd(session_name: &str) -> Command {
        let attach_command = match std::env::var("TMUX") {
            Ok(_) => "switch-client",
            Err(_) => "attach-session",
        };

        let tmux_friendly_name: String = session_name
            .chars()
            .map(|x| match x {
                '.' => '_',
                ':' => '_',
                _ => x,
            })
            .collect();

        shell!("tmux", attach_command, "-t", tmux_friendly_name)
    }
}
