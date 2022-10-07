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

    try_main(cli, config).unwrap_or_else(|message| {
        println!("error: {}", message);
        std::process::exit(1)
    })
}

fn try_main(cli: Cli, config: Config) -> Result<(), String> {
    match &cli.command {
        Commands::Clone { repo } => cmd::clone(repo, config),
        Commands::Tmux { repo } => cmd::tmux(repo, config),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
