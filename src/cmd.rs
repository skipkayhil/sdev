use std::fmt;
use std::io;
use std::io::Write;
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::repo::MaybeOwnedRepo;
use crate::repositories::git_repos::{FileSystemRepository, Repository};
use crate::Config;

macro_rules! println_shell {
    ($($arg:tt)*) => ({
        println!("\x1b[90m$ {}\x1b[0m", format_args!($($arg)*));
    })
}

macro_rules! shell {
    ($bin:expr, $($x:expr),* $(,)?) => {
        {
            let mut command = Command::new($bin);
            $(command.arg($x);)*
            PrintableCommand { command }
        }
    };
}

enum CmdError {
    IoError(String, io::Error),
}

impl From<CmdError> for String {
    fn from(e: CmdError) -> Self {
        match e {
            CmdError::IoError(cmd, io_e) => format!("failed to execute `{cmd}`: {io_e}"),
        }
    }
}

pub struct PrintableCommand {
    command: Command,
}

impl PrintableCommand {
    fn print_and_run(&mut self) -> Result<(), CmdError> {
        println_shell!("{}\n", self);

        self.run()
    }

    fn output(&mut self) -> Result<Output, CmdError> {
        self.command
            .output()
            .map_err(|e| CmdError::IoError(self.to_string(), e))
    }

    fn run(&mut self) -> Result<(), CmdError> {
        self.status().map(|_| {})
    }

    fn status(&mut self) -> Result<ExitStatus, CmdError> {
        self.command
            .status()
            .map_err(|e| CmdError::IoError(self.to_string(), e))
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

pub fn clone(repo_arg: &MaybeOwnedRepo, config: Config) -> Result<(), String> {
    let owner = repo_arg.owner().as_ref().unwrap_or(&config.user);
    let url = format!("git@github.com:{}/{}.git", owner, repo_arg.name());
    let path = config.root.join(owner).join(repo_arg.name());

    shell!("git", "clone", url, path)
        .print_and_run()
        .map_err(From::from)
}

pub fn tmux(config: Config) -> Result<(), String> {
    let repos = FileSystemRepository::new(config.root).fetch_all()?;

    let Some(repo_path) = fuzzy_select(repos.iter().map(|repo| repo.path()).collect())? else {
        return Ok(());
    };

    let repo = repos
        .iter()
        .find(|r| r.path() == repo_path)
        .expect("fzf should return an existing repo");

    let has_output = shell!("tmux", "has", "-t", format!("={}", repo.name())).output()?;

    if !has_output.status.success() {
        let (name, path) = (repo.name(), repo.path());
        shell!("tmux", "new-session", "-d", "-s", name, "-c", path).output()?;
    };

    tmux::attach_cmd(repo.name()).run().map_err(From::from)
}

pub enum FzfError {
    IoError(std::io::Error),
    PipeError,
}

impl From<FzfError> for String {
    fn from(e: FzfError) -> Self {
        match e {
            FzfError::IoError(io_e) => format!("error running fzf-tmux: {io_e}"),
            FzfError::PipeError => "error communicating with fzf".to_string(),
        }
    }
}

impl From<std::io::Error> for FzfError {
    fn from(e: std::io::Error) -> Self {
        FzfError::IoError(e)
    }
}

pub fn fuzzy_select<S>(options: Vec<S>) -> Result<Option<String>, FzfError>
where
    S: AsRef<str> + std::fmt::Display,
{
    let mut process = Command::new("fzf-tmux")
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    match process.stdin {
        Some(ref mut stdin) => {
            for option in options.iter() {
                writeln!(stdin, "{option}")?
            }
        }
        None => return Err(FzfError::PipeError),
    };

    let output = process.wait_with_output()?;

    let selected = output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string());

    Ok(selected)
}

pub mod tmux {
    use std::process::Command;

    use crate::cmd::PrintableCommand;

    pub fn attach_cmd(session_name: &str) -> PrintableCommand {
        let attach_command =
            std::env::var("TMUX").map_or_else(|_| "attach-session", |_| "switch-client");

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
