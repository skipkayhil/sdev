use std::io::Write;
use std::process::{Command, Stdio};

pub mod clone;
pub mod tmux;

pub enum FzfError {
    IoError(std::io::Error),
    PipeError,
}

impl From<FzfError> for String {
    fn from(e: FzfError) -> Self {
        match e {
            FzfError::IoError(io_e) => format!("error running fzf-tmux: {io_e}"),
            FzfError::PipeError => "error communicating with fzf".to_string(),
        }
    }
}

impl From<std::io::Error> for FzfError {
    fn from(e: std::io::Error) -> Self {
        FzfError::IoError(e)
    }
}

pub fn fuzzy_select<S, I>(options: I) -> Result<Option<String>, FzfError>
where
    S: AsRef<str> + std::fmt::Display,
    I: IntoIterator<Item = S>,
{
    let mut process = Command::new("fzf-tmux")
        .arg("-p")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    match process.stdin {
        Some(ref mut stdin) => {
            for option in options.into_iter() {
                writeln!(stdin, "{option}")?
            }
        }
        None => return Err(FzfError::PipeError),
    };

    let output = process.wait_with_output()?;

    let selected = output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string());

    Ok(selected)
}
