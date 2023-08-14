use bstr::BString;

use crate::dep::{Dep, MeetResult, MetResult, Reqs, UNMET};
use crate::shell;

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub struct Session {
    name: String,
    path: String,
}

impl Session {
    pub fn new<S: Into<String>>(name: S, path: S) -> Self {
        Session {
            name: name.into(),
            path: path.into(),
        }
    }
}

impl Dep for Session {
    fn met(&self) -> MetResult {
        let status = shell::new!("tmux", "has", "-t", format!("={}", self.name))
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
            &self.name,
            "-c",
            &self.path
        )
        .run(false)?;

        Ok(())
    }
}

pub struct Attach {
    name: String,
    path: String,
}

impl Attach {
    pub fn new<S: Into<String>>(name: S, path: S) -> Self {
        Attach {
            name: name.into(),
            path: path.into(),
        }
    }

    fn tmux_friendly_name(&self) -> String {
        self.name
            .chars()
            .map(|x| match x {
                '.' => '_',
                ':' => '_',
                _ => x,
            })
            .collect()
    }
}

impl Dep for Attach {
    fn met(&self) -> MetResult {
        if !in_tmux() {
            return UNMET;
        }

        let active_name: BString = shell::new!("tmux", "display-message", "-p", "\"#S\"")
            .output(false)?
            .stdout
            .into();

        Ok((active_name == self.name).into())
    }

    fn meet(&self) -> MeetResult {
        let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

        shell::new!("tmux", subcommand, "-t", self.tmux_friendly_name()).run(false)?;

        Ok(())
    }

    fn reqs_to_meet(&self) -> Reqs {
        vec![Box::new(Session::new(self.name.clone(), self.path.clone()))]
    }
}
