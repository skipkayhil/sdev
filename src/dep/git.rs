use crate::dep::{Dep, MeetResult, MetResult};
use crate::shell::{FG_GRAY, RESET};

use std::fs;
use std::path::PathBuf;

use gix::interrupt::IS_INTERRUPTED;
use gix::progress::Discard;
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
        println!("{}src{} {}", FG_GRAY, RESET, self.url);
        println!("{}dst{} {}", FG_GRAY, RESET, self.path.to_string_lossy());

        fs::create_dir_all(&self.path)?;

        let mut prepare_clone = gix::prepare_clone(self.url.clone(), &self.path)?;

        println!("cloning...");

        let (mut prepare_checkout, _) =
            prepare_clone.fetch_then_checkout(Discard, &IS_INTERRUPTED)?;

        println!("checking out...");

        prepare_checkout.main_worktree(Discard, &IS_INTERRUPTED)?;

        println!("done!");

        Ok(())
    }
}
