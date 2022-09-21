use clap::{Parser, Subcommand};
use std::path::Path;

mod cmd;
mod config;
mod repo;

use crate::config::Config;
use crate::repo::MaybeOwnedRepo;

#[derive(Parser)]
#[command(version, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clones a repo into a pre-determined folder
    Clone { repo: MaybeOwnedRepo },
    /// Attaches to a tmux session for the repo (and creates it if necessary)
    #[command(alias("t"))]
    Tmux { repo: MaybeOwnedRepo },
}

fn main() {
    let cli = Cli::parse();

    let config = Config {
        user: "skipkayhil".to_string(),
        root: Path::new(&std::env::var("HOME").expect("unknown HOME directory"))
            .join("src")
            .join("github.com"),
    };

    let status = match &cli.command {
        Commands::Clone { repo } => {
            let parsed_repo = repo.unwrap_or_else(|_| config.user.clone());

            cmd::run_printable(cmd::git::clone_cmd(&parsed_repo, &config))
        }
        Commands::Tmux { repo } => {
            let parsed_repo = repo.unwrap_or_else(|name| {
                cmd::find::owner(name, &config).unwrap_or_else(|error| {
                    println!("error: {}", error);
                    std::process::exit(1);
                })
            });

            if !cmd::tmux::session_exists(&parsed_repo) {
                cmd::tmux::new_session(&parsed_repo, &config);
            }

            cmd::run_printable(cmd::tmux::attach_cmd(&parsed_repo))
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
