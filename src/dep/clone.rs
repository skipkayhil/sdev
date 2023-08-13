use crate::dep::{Dep, DepResult};
use crate::shell;
use std::path::PathBuf;

pub struct Clone {
    url: String,
    path: PathBuf,
}

impl Clone {
    pub fn new(url: String, path: PathBuf) -> Self {
        Clone { url, path }
    }
}

impl Dep for Clone {
    fn met(&self) -> DepResult {
        Ok(self.path.join(".git").is_dir().into())
    }

    fn meet(&self) -> bool {
        shell::new!("git", "clone", &self.url, &self.path)
            .run(true)
            .is_ok()
    }
}
