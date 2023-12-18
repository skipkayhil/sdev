use clap::{Args, Parser, Subcommand};
use std::path::Path;

mod cmd;
mod config;
mod dep;
mod repo;
mod repositories;
mod shell;

use crate::config::Config;
use crate::repo::GitRepoSource;

#[derive(Parser)]
#[command(version, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clones a git repository into a standardized path
    Clone {
        repo: GitRepoSource,
    },
    Open(OpenArgs),
    /// Fuzzy attach to a repo's tmux session (and creates it if necessary)
    #[command(alias("t"))]
    Tmux,
}

#[derive(Debug, Args)]
struct OpenArgs {
    #[command(subcommand)]
    command: OpenCommands,
}

#[derive(Debug, Subcommand)]
enum OpenCommands {
    /// Open the New Pull Request form for the current branch
    Pr { target: Option<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config {
        host: "github.com".to_string(),
        root: Path::new(&std::env::var("HOME").expect("unknown HOME directory")).join("src"),
        user: "skipkayhil".to_string(),
    };

    match &cli.command {
        Commands::Clone { repo } => cmd::clone::run(repo, &config),
        Commands::Open(open) => match &open.command {
            OpenCommands::Pr { target } => cmd::open::pr::run(target),
        },
        Commands::Tmux => cmd::tmux::run(config),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
