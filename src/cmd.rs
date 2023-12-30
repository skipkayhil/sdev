use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

pub mod clone;
pub mod open;
pub mod tmux;

#[derive(thiserror::Error, Debug)]
pub enum FzfError {
    #[error("error getting output from fzf-tmux")]
    Output(#[source] io::Error),
    #[error("error creating pipe to fzf-tmux")]
    Pipe,
    #[error("error running fzf-tmux")]
    Spawn(#[source] io::Error),
    #[error("error writing options to fzf-tmux")]
    Write(#[source] io::Error),
}

pub fn fuzzy_select<'a, T>(options: T) -> Result<Option<String>, FzfError>
where
    T: IntoIterator<Item = &'a OsStr>,
{
    let mut process = Command::new("fzf-tmux")
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(FzfError::Spawn)?;

    match process.stdin {
        Some(ref mut stdin) => {
            for option in options.into_iter() {
                stdin
                    .write(option.as_encoded_bytes())
                    .map_err(FzfError::Write)?;
                writeln!(stdin).map_err(FzfError::Write)?;
            }
        }
        None => return Err(FzfError::Pipe),
    };

    let output = process.wait_with_output().map_err(FzfError::Output)?;

    let selected = output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string());

    Ok(selected)
}
