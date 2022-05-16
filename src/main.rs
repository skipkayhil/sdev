use clap::{Parser, Subcommand};
use std::env;
use std::path::Path;
use std::process::Command;

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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone { repo } => {
            let url = repo.to_url();
            let path = Path::new(&env::var("HOME").unwrap()).join(&repo.to_relative_path());

            println!(
                "\x1b[90m$ git clone \"{}\" \"{}\"\x1b[0m",
                url,
                path.display()
            );

            let status = Command::new("git")
                .arg("clone")
                .arg(&url)
                .arg(&path)
                .status()
                .expect("failed to execute git");

            if !status.success() {
                std::process::exit(status.code().unwrap());
            }
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
