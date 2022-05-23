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

    let command = match &cli.command {
        Commands::Clone { repo } => SdevCommand::clone(repo),
    };

    let status = command.run();

    if !status.success() {
        std::process::exit(status.code().unwrap());
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
