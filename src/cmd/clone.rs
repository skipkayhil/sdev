use std::process::Command;

use crate::cmd::PrintableCommand;
use crate::repo::MaybeOwnedRepo;
use crate::Config;

pub fn run(repo_arg: &MaybeOwnedRepo, config: Config) -> Result<(), String> {
    let owner = repo_arg.owner().as_ref().unwrap_or(&config.user);
    let url = format!("git@github.com:{}/{}.git", owner, repo_arg.name());
    let path = config.root.join(owner).join(repo_arg.name());

    shell!("git", "clone", url, path)
        .print_and_run()
        .map_err(From::from)
}
