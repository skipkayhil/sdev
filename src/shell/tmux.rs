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

#[derive(Clone)]
pub struct Session {
    name: SessionName,
}

impl Session {
    pub fn find_or_create_in<S: Into<SessionName>>(
        name: S,
        path: &Path,
    ) -> Result<Self, shell::ShellError> {
        let session_name = name.into();

        match has(&session_name)? {
            Some(session) => Ok(session),
            None => Ok(new_session(&session_name, path)?),
        }
    }

    pub fn attach_or_switch(&self) -> Result<(), shell::ShellError> {
        attach_or_switch(&self.name)
    }

    pub fn name_str(&self) -> &str {
        &self.name.0
    }
}

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

fn attach_or_switch(name: &SessionName) -> Result<(), shell::ShellError> {
    let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

    shell::new!("tmux", subcommand, "-t", &name.0).run(false)
}

fn has(name: &SessionName) -> Result<Option<Session>, shell::ShellError> {
    if shell::new!("tmux", "has", "-t", format!("={}", name.0))
        // output instead of status to intercept stdout
        .output(false)?
        .status
        .success()
    {
        Ok(Some(Session { name: name.clone() }))
    } else {
        Ok(None)
    }
}

pub fn list_sessions() -> Result<Vec<Session>, anyhow::Error> {
    let raw_output = shell::new!("tmux", "list-sessions", "-F", "#{session_name}").output(false)?;

    let parsed_output = String::from_utf8(raw_output.stdout)?;

    Ok(parsed_output
        .lines()
        .map(|s| Session {
            name: SessionName::from(s),
        })
        .collect())
}

fn new_session(name: &SessionName, path: &Path) -> Result<Session, shell::ShellError> {
    shell::new!("tmux", "new-session", "-d", "-s", &name.0, "-c", &path).run(false)?;

    Ok(Session { name: name.clone() })
}
