use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod cmd;
mod config;
mod dep;
mod repo;
mod shell;
mod ui;

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
    /// Clone a git repository into a standardized path
    Clone { repo: GitRepoSource },
    /// Open a link for the current repository
    #[command(alias("o"))]
    Open(OpenArgs),
    /// Fuzzy attach to a repository's tmux session (creating it if necessary)
    #[command(alias("t"))]
    Tmux {
        #[arg(long, default_value_t = cmd::tmux::Mode::Repos, value_enum)]
        mode: cmd::tmux::Mode,
    },
}

#[derive(Debug, Args)]
struct OpenArgs {
    #[command(subcommand)]
    command: OpenCommands,
}

#[derive(Debug, Subcommand)]
enum OpenCommands {
    /// Open the file in the remote's UI
    File { path: PathBuf },
    /// Open the New Pull Request form for the current branch
    Pr { target: Option<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config {
        host: "github.com".to_string(),
        root: std::env::home_dir()
            .expect("unknown HOME directory")
            .join("src"),
        user: "skipkayhil".to_string(),
    };

    match &cli.command {
        Commands::Clone { repo } => cmd::clone::run(repo, &config),
        Commands::Open(open) => match &open.command {
            OpenCommands::File { path } => cmd::open::file::run(path),
            OpenCommands::Pr { target } => cmd::open::pr::run(target),
        },
        Commands::Tmux { mode } => cmd::tmux::run(mode, config),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
