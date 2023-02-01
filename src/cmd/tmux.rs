use std::process::Command;

use crate::cmd::{fuzzy_select, PrintableCommand};
use crate::repositories::git_repos::{FileSystemRepository, Repository};
use crate::Config;

pub fn run(config: Config) -> Result<(), String> {
    let repos = FileSystemRepository::new(config.root).fetch_all()?;

    let Some(repo_path) = fuzzy_select(repos.iter().map(|repo| repo.path()).collect())? else {
        return Ok(());
    };

    let repo = repos
        .iter()
        .find(|r| r.path() == repo_path)
        .expect("fzf should return an existing repo");

    let has_output = shell!("tmux", "has", "-t", format!("={}", repo.name())).output()?;

    if !has_output.status.success() {
        let (name, path) = (repo.name(), repo.path());
        shell!("tmux", "new-session", "-d", "-s", name, "-c", path).output()?;
    };

    attach_cmd(repo.name()).run().map_err(From::from)
}

fn attach_cmd(session_name: &str) -> PrintableCommand {
    let attach_command =
        std::env::var("TMUX").map_or_else(|_| "attach-session", |_| "switch-client");

    let tmux_friendly_name: String = session_name
        .chars()
        .map(|x| match x {
            '.' => '_',
            ':' => '_',
            _ => x,
        })
        .collect();

    shell!("tmux", attach_command, "-t", tmux_friendly_name)
}
