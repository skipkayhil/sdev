use crate::dep::Dep;
use crate::shell;

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
    fn met(&self) -> bool {
        shell::new!("tmux", "has", "-t", format!("={}", self.name))
            // output instead of status to intercept stdout
            .output(false)
            .is_ok_and(|x| x.status.success())
    }

    fn meet(&self) -> bool {
        shell::new!(
            "tmux",
            "new-session",
            "-d",
            "-s",
            &self.name,
            "-c",
            &self.path
        )
        .run(false)
        .is_ok()
    }
}
