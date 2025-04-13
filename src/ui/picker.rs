use std::path::Path;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

use nucleo::Matcher;
use nucleo::{
    Config, Nucleo,
    pattern::{CaseMatching, Normalization},
};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{ListState, StatefulWidget};
use ratatui::{
    prelude::{Color, Line, Span, Style, Stylize},
    widgets::{Block, Borders, List, ListDirection},
};

use crate::repo::GitRepo;

pub struct PickerState {
    nucleo: Nucleo<GitRepo>,
    selected: u32,
    pub state: ListState,
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

    pub fn dec_selection(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        // TODO: unwrap because List uses usize, custom List will fix that
        self.state.select(Some(self.selected.try_into().unwrap()));
    }

    pub fn inc_selection(&mut self) {
        let incremented_selection = self.selected.saturating_add(1);

        if self.nucleo.snapshot().matched_item_count() > incremented_selection {
            self.selected = self.selected.saturating_add(1);
            // TODO: unwrap because List uses usize, custom List will fix that
            self.state.select(Some(self.selected.try_into().unwrap()));
        }
    }

    pub fn get_selected(&self) -> Option<GitRepo> {
        self.nucleo
            .snapshot()
            .get_matched_item(self.selected)
            .map(|item| item.data)
            .cloned()
    }

    pub fn push(&mut self, repo: GitRepo, root: &Path) {
        self.nucleo.injector().push(repo, |repo_ref, dst| {
            dst[0] = repo_ref.relative_path(root).to_string_lossy().into()
        });
    }

    pub fn tick(&mut self) {
        let status = self.nucleo.tick(10);

        if status.changed {
            self.selected = self.selected.min(
                self.nucleo
                    .snapshot()
                    .matched_item_count()
                    .saturating_sub(1),
            );
            // TODO: unwrap because List uses usize, custom List will fix that
            self.state.select(Some(self.selected.try_into().unwrap()));
        }
    }
}

impl Default for PickerState {
    fn default() -> Self {
        let nucleo = Nucleo::<GitRepo>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let state = ListState::default().with_selected(Some(0));

        Self {
            nucleo,
            selected: 0,
            state,
        }
    }
}

pub struct Picker {}

const PADDED_CHEVRON: &str = "> ";
static MATCHER: LazyLock<Mutex<Matcher>> = LazyLock::new(|| Mutex::new(Matcher::default()));

impl StatefulWidget for Picker {
    type State = PickerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut matcher = MATCHER.lock().unwrap();
        let mut col_indices = Vec::new();

        let snap = state.nucleo.snapshot();
        let matched_paths: Vec<Line> = snap
            .matched_items(0..snap.matched_item_count().min(area.height.into()))
            .map(|item| {
                let relative_path = item.matcher_columns[0].slice(..);

                snap.pattern().column_pattern(0).indices(
                    relative_path,
                    &mut matcher,
                    &mut col_indices,
                );

                col_indices.dedup();
                col_indices.sort_unstable();

                let mut styled_path = Line::from(
                    relative_path
                        .chars()
                        .map(|c| c.to_string().into())
                        .collect::<Vec<Span>>(),
                );

                col_indices.drain(..).for_each(|i| {
                    let index: usize = i.try_into().expect("you really have a path that long?");
                    styled_path.spans[index] = styled_path.spans[index].clone().red();
                });

                styled_path
            })
            .collect();

        let path_list = List::new(matched_paths)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::new().dark_gray()),
            )
            .highlight_symbol(PADDED_CHEVRON)
            .highlight_style(Style::new().bold().bg(Color::Indexed(18)))
            .direction(ListDirection::BottomToTop);

        path_list.render(area, buf, &mut state.state);
    }
}
