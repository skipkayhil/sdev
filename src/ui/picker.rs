use std::path::Path;
use std::sync::Arc;

use nucleo::{
    Config, Nucleo,
    pattern::{CaseMatching, Normalization},
};

use crate::repo::GitRepo;

pub struct PickerState {
    nucleo: Nucleo<GitRepo>,
}

impl PickerState {
    pub fn pop_char(&mut self, search: &str) {
        self.nucleo
            .pattern
            .reparse(0, search, CaseMatching::Smart, Normalization::Smart, false);
    }

    pub fn push_char(&mut self, search: &str) {
        self.nucleo
            .pattern
            .reparse(0, search, CaseMatching::Smart, Normalization::Smart, true);
    }

    pub fn matched_item_count(&self) -> u32 {
        self.nucleo.snapshot().matched_item_count()
    }

    pub fn get_matched_item(&self, i: u32) -> Option<&GitRepo> {
        self.nucleo
            .snapshot()
            .get_matched_item(i)
            .map(|item| item.data)
    }

    pub fn push(&mut self, repo: GitRepo, root: &Path) {
        self.nucleo.injector().push(repo, |repo_ref, dst| {
            dst[0] = repo_ref.relative_path(root).to_string_lossy().into()
        });
    }

    // tmp until Picker renders itself
    pub fn snapshot(&self) -> &nucleo::Snapshot<GitRepo> {
        self.nucleo.snapshot()
    }

    pub fn tick(&mut self) -> nucleo::Status {
        self.nucleo.tick(10)
    }
}

impl Default for PickerState {
    fn default() -> Self {
        let nucleo = Nucleo::<GitRepo>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);

        Self { nucleo }
    }
}
