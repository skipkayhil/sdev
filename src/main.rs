use clap::{Parser, Subcommand};
use std::fmt;
use std::process::{Command, ExitStatus};

mod cmd;
mod repo;

use crate::repo::Repo;

macro_rules! println_shell {
    ($($arg:tt)*) => ({
        println!("\x1b[90m$ {}\x1b[0m", format_args!($($arg)*));
    })
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clones a repo into a pre-determined folder
    Clone {
        #[clap(parse(try_from_str))]
        repo: Repo,
    },
    /// Attaches to a tmux session for the repo (and creates it if necessary)
    #[clap(alias("t"))]
    Tmux {
        #[clap(parse(try_from_str))]
        repo: Repo,
    },
}

struct SdevCommand {
    command: Command,
}

impl SdevCommand {
    fn new(command: Command) -> Self {
        Self { command }
    }

    fn run(&mut self) -> ExitStatus {
        println_shell!("{}\n", self);

        self.command
            .status()
            .unwrap_or_else(|_| panic!("failed to execute {:?}", self.command.get_program()))
    }
}

impl fmt::Display for SdevCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.command.get_program().to_str().unwrap())?;

        for arg in self.command.get_args() {
            write!(f, " {}", arg.to_str().unwrap())?;
        }

        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    let status = match &cli.command {
        Commands::Clone { repo } => SdevCommand::new(cmd::git::clone_cmd(repo)).run(),
        Commands::Tmux { repo } => {
            if !cmd::tmux::session_exists(repo) {
                cmd::tmux::new_session(repo);
            }

            SdevCommand::new(cmd::tmux::attach_cmd(repo)).run()
        }
    };

    if !status.success() {
        std::process::exit(status.code().unwrap());
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
