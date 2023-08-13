use std::path::PathBuf;

use crate::cmd::fuzzy_select;
use crate::dep::{tmux, Dep};
use crate::repo::GitRepo;
use crate::repositories::git_repos::{CachingRepository, FileSystemRepository, Repository};
use crate::Config;

pub fn run(config: Config) -> Result<(), String> {
    let Some(repo) = fuzzy_select_repository(config.root)? else {
        return Ok(());
    };

    tmux::Attach::new(repo.name(), repo.path()).process();

    Ok(())
}

fn fuzzy_select_repository(root: PathBuf) -> Result<Option<GitRepo>, String> {
    let mut repository = CachingRepository::new(FileSystemRepository::new(root));

    let all_paths = repository.fetch_all()?;
    let maybe_path = fuzzy_select(all_paths.iter().map(|repo| repo.path()))?;

    Ok(maybe_path.and_then(|path| repository.fetch_one_from_cache(&path)))
}
