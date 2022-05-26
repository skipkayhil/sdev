use clap::{Parser, Subcommand};

mod cmd;
mod repo;

use crate::repo::Repo;

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

fn main() {
    let cli = Cli::parse();

    let status = match &cli.command {
        Commands::Clone { repo } => cmd::run_printable(cmd::git::clone_cmd(repo)),
        Commands::Tmux { repo } => {
            if !cmd::tmux::session_exists(repo) {
                cmd::tmux::new_session(repo);
            }

            cmd::run_printable(cmd::tmux::attach_cmd(repo))
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
