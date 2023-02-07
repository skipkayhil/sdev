use std::path::PathBuf;
use std::process::Command;

use crate::cmd::{fuzzy_select, PrintableCommand};
use crate::repo::GitRepo;
use crate::repositories::git_repos::{CachingRepository, FileSystemRepository, Repository};
use crate::Config;

pub fn run(config: Config) -> Result<(), String> {
    let Some(repo) = fuzzy_select_repository(config.root)? else {
        return Ok(());
    };

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

fn fuzzy_select_repository(root: PathBuf) -> Result<Option<GitRepo>, String> {
    let mut repository = CachingRepository::new(FileSystemRepository::new(root));

    let all_paths = repository.fetch_all()?;
    let maybe_path = fuzzy_select(all_paths.iter().map(|repo| repo.path()))?;

    Ok(maybe_path.and_then(|path| repository.fetch_one_from_cache(&path)))
}
