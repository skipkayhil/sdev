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
    program: String,
    args: Vec<String>,
}

impl SdevCommand {
    fn clone(repo: &Repo) -> Self {
        Self {
            program: "git".to_string(),
            args: vec![
                "clone".to_string(),
                repo.to_url(),
                repo.to_path_with_base(&env::var("HOME").expect("unknown HOME directory"))
                    .display()
                    .to_string(),
            ],
        }
    }

    fn tmux(repo: &Repo, command: &str) -> Self {
        Self {
            program: "tmux".to_string(),
            args: vec![
                command.to_string(),
                "-t".to_string(),
                repo.name().to_string(),
            ],
        }
    }

    fn run(&self) -> ExitStatus {
        println_shell!("{}\n", &self);

        let mut command = Command::new(&self.program);

        for arg in self.args.iter() {
            command.arg(&arg);
        }

        command
            .status()
            .unwrap_or_else(|_| panic!("failed to execute {}", &self.program))
    }
}

impl fmt::Display for SdevCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.program)?;

        for arg in self.args.iter() {
            write!(f, " {}", arg)?;
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
