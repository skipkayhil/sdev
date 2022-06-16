use clap::{Parser, Subcommand};
use std::path::Path;

mod cmd;
mod config;
mod repo;

use crate::config::Config;
use crate::repo::{MaybeOwnedRepo, Repo};

#[derive(Parser)]
#[clap(disable_help_subcommand = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clones a repo into a pre-determined folder
    Clone { repo: MaybeOwnedRepo },
    /// Attaches to a tmux session for the repo (and creates it if necessary)
    #[clap(alias("t"))]
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
            let parsed_repo = Repo::from_str_with_fallback(repo, &config.user);

            cmd::run_printable(cmd::git::clone_cmd(&parsed_repo, &config))
        }
        Commands::Tmux { repo } => {
            let parsed_repo = Repo::from_str_with_fallback(repo, &config.user);

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
