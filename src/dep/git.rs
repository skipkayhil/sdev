use crate::dep::{Dep, MeetResult, MetResult};
use crate::shell;

use std::path::PathBuf;

use gix::Url;

pub struct Clone {
    url: Url,
    path: PathBuf,
}

impl Clone {
    pub fn new(url: Url, path: PathBuf) -> Self {
        Clone { url, path }
    }
}

impl Dep for Clone {
    fn met(&self) -> MetResult {
        Ok(self.path.join(".git").is_dir().into())
    }

    fn meet(&self) -> MeetResult {
        shell::new!("git", "clone", &self.url.to_string(), &self.path).run(true)?;

        Ok(())
    }
}
