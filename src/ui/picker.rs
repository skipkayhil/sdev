use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

use nucleo::Matcher;
use nucleo::{
    Config, Nucleo, Utf32String,
    pattern::{CaseMatching, Normalization},
};
use ratatui::{
    prelude::{Buffer, Line, Rect, Span, Style, Stylize},
    widgets::{Block, Borders, Widget},
};

const PADDED_CHEVRON: &str = "> ";
static MATCHER: LazyLock<Mutex<Matcher>> = LazyLock::new(|| Mutex::new(Matcher::default()));

type FormatFn<T, D> = fn(&T, &D) -> Utf32String;

pub struct Picker<T: Clone + Send + Sync + 'static, D> {
    nucleo: Nucleo<T>,
    selected: u32,
    formatter: FormatFn<T, D>,
    data: D,
}

impl<T: Clone + Send + Sync + 'static, D> Picker<T, D> {
    pub fn new(formatter: FormatFn<T, D>, data: D) -> Self {
        let nucleo = Nucleo::<T>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);

        Self {
            nucleo,
            selected: 0,
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
    }

    pub fn inc_selection(&mut self) {
        let incremented_selection = self.selected.saturating_add(1);

        if self.nucleo.snapshot().matched_item_count() > incremented_selection {
            self.selected = incremented_selection;
        }
    }

    pub fn select(&mut self, i: u32) {
        if self.nucleo.snapshot().matched_item_count() > i {
            self.selected = i;
        } else {
            self.selected = self.nucleo.snapshot().matched_item_count() - 1
        }
    }

    pub fn selected(&self) -> u32 {
        self.selected
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
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.tick();

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().dark_gray());
        let inner_area = block.inner(area);

        let mut matcher = MATCHER.lock().unwrap();
        let mut col_indices = Vec::new();

        let snap = &self.nucleo.snapshot();

        let min_displayed = 0;
        let max_displayed = snap.matched_item_count().min(inner_area.height.into());

        Widget::render(block, area, buf);

        let mut current_y = inner_area.bottom() - 1;
        let selected_y = current_y.saturating_sub(self.selected.try_into().unwrap_or(u16::MAX));

        snap.matched_items(min_displayed..max_displayed)
            .for_each(|item| {
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
                        .map(|c| {
                            if current_y == selected_y {
                                c.to_string().into()
                            } else {
                                c.to_string().dark_gray()
                            }
                        })
                        .collect::<Vec<Span>>(),
                );

                col_indices.drain(..).for_each(|i| {
                    let index: usize = i.try_into().expect("you really have a string that long?");
                    styled_string.spans[index] = styled_string.spans[index].clone().red();
                });

                if current_y == selected_y {
                    let selected_indicator_rect = Rect {
                        x: 0,
                        y: selected_y,
                        height: 1,
                        width: 2,
                    };

                    Widget::render(PADDED_CHEVRON, selected_indicator_rect, buf);

                    let rect = Rect {
                        x: 2,
                        y: current_y,
                        height: 1,
                        width: inner_area.width,
                    };

                    Widget::render(styled_string, rect, buf);
                } else {
                    let rect = Rect {
                        x: 2,
                        y: current_y,
                        height: 1,
                        width: styled_string.width().try_into().unwrap(),
                    };

                    Widget::render(styled_string, rect, buf);
                }

                current_y = current_y.saturating_sub(1);
            });
    }
}
