use std::path::PathBuf;

use crate::dep::{Dep, MeetResult, MetResult};
use crate::shell::tmux;

pub struct Session {
    name: String,
    path: PathBuf,
}

impl Session {
    pub fn process(name: String, path: PathBuf) -> MeetResult {
        Session { name, path }.process()
    }
}

impl Dep for Session {
    fn met(&self) -> MetResult {
        Ok(tmux::has(&*self.name)?.into())
    }

    fn meet(&self) -> MeetResult {
        Ok(tmux::new_session(&*self.name, &self.path)?)
    }
}
