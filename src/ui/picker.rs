use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

use nucleo::Matcher;
use nucleo::{
    Config, Nucleo, Utf32String,
    pattern::{CaseMatching, Normalization},
};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{ListState, StatefulWidget};
use ratatui::{
    prelude::{Color, Line, Span, Style, Stylize},
    widgets::{Block, Borders, List, ListDirection},
};

const PADDED_CHEVRON: &str = "> ";
static MATCHER: LazyLock<Mutex<Matcher>> = LazyLock::new(|| Mutex::new(Matcher::default()));

type FormatFn<T, D> = fn(&T, &D) -> Utf32String;

pub struct Picker<T: Clone + Send + Sync + 'static, D> {
    nucleo: Nucleo<T>,
    selected: u32,
    pub state: ListState,
    formatter: FormatFn<T, D>,
    data: D,
}

impl<T: Clone + Send + Sync + 'static, D> Picker<T, D> {
    pub fn new(formatter: FormatFn<T, D>, data: D) -> Self {
        let nucleo = Nucleo::<T>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let state = ListState::default().with_selected(Some(0));

        Self {
            nucleo,
            selected: 0,
            state,
            formatter,
            data,
        }
    }

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

    pub fn get_selected(&self) -> Option<T> {
        self.nucleo
            .snapshot()
            .get_matched_item(self.selected)
            .map(|item| item.data)
            .cloned()
    }

    pub fn push(&mut self, t: T) {
        self.nucleo
            .injector()
            .push(t, |t_ref, dst| dst[0] = (self.formatter)(t_ref, &self.data));
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

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let mut matcher = MATCHER.lock().unwrap();
        let mut col_indices = Vec::new();

        let snap = &self.nucleo.snapshot();
        let matches: Vec<Line> = snap
            .matched_items(0..snap.matched_item_count().min(area.height.into()))
            .map(|item| {
                let matched_string = item.matcher_columns[0].slice(..);

                snap.pattern().column_pattern(0).indices(
                    matched_string,
                    &mut matcher,
                    &mut col_indices,
                );

                col_indices.dedup();
                col_indices.sort_unstable();

                let mut styled_string = Line::from(
                    matched_string
                        .chars()
                        .map(|c| c.to_string().into())
                        .collect::<Vec<Span>>(),
                );

                col_indices.drain(..).for_each(|i| {
                    let index: usize = i.try_into().expect("you really have a string that long?");
                    styled_string.spans[index] = styled_string.spans[index].clone().red();
                });

                styled_string
            })
            .collect();

        let match_list = List::new(matches)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::new().dark_gray()),
            )
            .highlight_symbol(PADDED_CHEVRON)
            .highlight_style(Style::new().bold().bg(Color::Indexed(18)))
            .direction(ListDirection::BottomToTop);

        match_list.render(area, buf, &mut self.state);
    }
}
