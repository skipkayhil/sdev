use std::path::PathBuf;

use crate::dep::{Dep, MeetResult, MetResult};
use crate::shell;

#[derive(Clone)]
pub struct SessionName(pub String);

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

pub struct Session {
    name: SessionName,
    path: PathBuf,
}

impl Session {
    pub fn new<S: Into<SessionName>>(name: S, path: PathBuf) -> Self {
        Session {
            name: name.into(),
            path,
        }
    }
}

impl Dep for Session {
    fn met(&self) -> MetResult {
        let status = shell::new!("tmux", "has", "-t", format!("={}", self.name.0))
            // output instead of status to intercept stdout
            .output(false)?
            .status
            .success();

        Ok(status.into())
    }

    fn meet(&self) -> MeetResult {
        shell::new!(
            "tmux",
            "new-session",
            "-d",
            "-s",
            &self.name.0,
            "-c",
            &self.path
        )
        .run(false)?;

        Ok(())
    }
}
