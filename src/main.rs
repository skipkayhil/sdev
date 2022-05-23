use clap::{Parser, Subcommand};
use std::env;
use std::fmt;
use std::process::{Command, ExitStatus};

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
    Tmux {
        #[clap(parse(try_from_str))]
        repo: Repo,
    },
}

struct SdevCommand {
    command: Command,
}

impl SdevCommand {
    fn clone(repo: &Repo) -> Self {
        let mut command = Command::new("git");

        command
            .arg("clone")
            .arg(repo.to_url())
            .arg(repo.to_path_with_base(&env::var("HOME").expect("unknown HOME directory")));

        Self { command }
    }

    fn tmux(repo: &Repo, attach_command: &str) -> Self {
        let mut command = Command::new("tmux");

        command.arg(attach_command).arg("-t").arg(repo.name());

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
        Commands::Clone { repo } => SdevCommand::clone(repo).run(),
        Commands::Tmux { repo } => {
            let session_exists = Command::new("tmux")
                .arg("has")
                .arg("-t")
                .arg(repo.name())
                .output()
                .expect("failed to execute 'tmux has'")
                .status
                .success();

            if !session_exists {
                Command::new("tmux")
                    .arg("new-session")
                    .arg("-d")
                    .arg("-s")
                    .arg(repo.name())
                    .arg("-c")
                    .arg(repo.to_path_with_base(&env::var("HOME").expect("unknown HOME directory")))
                    .output()
                    .expect("failed to execute 'tmux new-session'");
            }

            let attach_command = match env::var("TMUX") {
                Ok(_) => "switch-client",
                Err(_) => "attach-session",
            };

            SdevCommand::tmux(repo, attach_command).run()
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
