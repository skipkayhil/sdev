use std::path::Path;

use crate::shell;

#[derive(Clone)]
pub struct SessionName(String);

impl From<&str> for SessionName {
    fn from(value: &str) -> Self {
        Self(
            value
                .chars()
                .map(|x| match x {
                    '.' => '_',
                    ':' => '_',
                    _ => x,
                })
                .collect(),
        )
    }
}

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn attach_or_switch<S: Into<SessionName>>(name: S) -> Result<(), shell::ShellError> {
    let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

    shell::new!("tmux", subcommand, "-t", name.into().0).run(false)
}

pub fn has<S: Into<SessionName>>(name: S) -> Result<bool, shell::ShellError> {
    let status = shell::new!("tmux", "has", "-t", format!("={}", name.into().0))
        // output instead of status to intercept stdout
        .output(false)?
        .status
        .success();

    Ok(status)
}

pub fn list_sessions() -> Result<Vec<String>, anyhow::Error> {
    let raw_output = shell::new!("tmux", "list-sessions", "-F", "#{session_name}").output(false)?;

    let parsed_output = String::from_utf8(raw_output.stdout)?;

    Ok(parsed_output.lines().map(str::to_string).collect())
}

pub fn new_session<S: Into<SessionName>>(name: S, path: &Path) -> Result<(), shell::ShellError> {
    shell::new!(
        "tmux",
        "new-session",
        "-d",
        "-s",
        name.into().0,
        "-c",
        &path
    )
    .run(false)
}
