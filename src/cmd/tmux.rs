use std::path::PathBuf;

use crate::cmd::{fuzzy_select, FzfError};
use crate::dep::{tmux, Dep};
use crate::repo::GitRepo;
use crate::repositories::git_repos::{
    CachingRepository, FetchAllError, FileSystemRepository, Repository,
};
use crate::shell;
use crate::Config;

#[derive(thiserror::Error, Debug)]
#[error("error running tmux command")]
enum TmuxError {
    Fetch(#[source] FetchAllError),
    Fzf(#[source] FzfError),
}

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

pub fn run(config: Config) -> anyhow::Result<()> {
    let Some(repo) = fuzzy_select_repository(config.root)? else {
        return Ok(());
    };

    let name = tmux::SessionName::from(repo.name());

    tmux::Session::new(name.clone(), repo.path()).process()?;

    let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

    shell::new!("tmux", subcommand, "-t", &name.0).run(false)?;

    Ok(())
}

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

fn fuzzy_select_repository(root: PathBuf) -> Result<Option<GitRepo>, TmuxError> {
    let mut repository = CachingRepository::new(FileSystemRepository::new(root));

    let all_paths = repository.fetch_all().map_err(TmuxError::Fetch)?;
    let maybe_path =
        fuzzy_select(all_paths.iter().map(|repo| repo.path())).map_err(TmuxError::Fzf)?;

    Ok(maybe_path.and_then(|path| repository.fetch_one_from_cache(&path)))
}
