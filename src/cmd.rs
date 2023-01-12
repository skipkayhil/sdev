use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

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

pub fn tmux(config: Config) -> Result<(), String> {
    let repos = FileSystemRepository::new(config.root).fetch_all()?;

    let Some(repo_path) = fuzzy_select(repos.iter().map(|repo| repo.path()).collect())? else {
        return Ok(());
    };

    let repo = repos
        .iter()
        .find(|r| r.path() == repo_path)
        .expect("fzf should return an existing repo");

    let mut command = shell!("tmux", "has", "-t", format!("={}", repo.name()));

    match command.output() {
        Err(_) => return Err(PrintableCommand { command }.error_message()),
        Ok(output) if !output.status.success() => tmux::create_session(repo)?,
        _ => {}
    }

    match tmux::attach_cmd(repo.name()).output() {
        Err(_) => Err(PrintableCommand { command }.error_message()),
        _ => Ok(()),
    }
}

pub enum FzfError {
    IoError(std::io::Error),
    PipeError,
}

impl From<FzfError> for String {
    fn from(e: FzfError) -> Self {
        match e {
            FzfError::IoError(io_e) => format!("error running fzf-tmux: {}", io_e),
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
    let mut process = shell!("fzf-tmux", "-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    match process.stdin {
        Some(ref mut stdin) => {
            for option in options.iter() {
                writeln!(stdin, "{}", option)?
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
    use crate::repo::GitRepo;

    pub fn create_session(repo: &GitRepo) -> Result<(), String> {
        let mut command = shell!(
            "tmux",
            "new-session",
            "-d",
            "-s",
            repo.name(),
            "-c",
            repo.path(),
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
