use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Color, Line, Span, Style, Stylize},
    widgets::{Block, Borders, List, ListDirection},
    Frame,
};

use crate::cmd::tmux::{App, MATCHER};

const PADDED_CHEVRON: &str = "> ";

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout =
        Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(frame.area());

    let mut matcher = MATCHER.lock().unwrap();
    let mut col_indices = Vec::new();

    let snap = app.nucleo.snapshot();
    let matched_paths: Vec<Line> = snap
        .matched_items(0..snap.matched_item_count().min(layout[0].height.into()))
        .map(|item| {
            snap.pattern().column_pattern(0).indices(
                item.matcher_columns[0].slice(..),
                &mut matcher,
                &mut col_indices,
            );

            col_indices.dedup();
            col_indices.sort_unstable();

            let mut styled_path = Line::from(
                item.data
                    .relative_path(&app.config.root)
                    .to_string_lossy()
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
        .highlight_style(Style::new().bold().bg(Color::Indexed(19)))
        .direction(ListDirection::BottomToTop);

    frame.render_stateful_widget(path_list, layout[0], &mut app.state);

    let prompt = Line::from(vec![
        PADDED_CHEVRON.bold().magenta(),
        app.search.clone().bold(),
    ]);

    frame.render_widget(prompt, layout[1]);

    // TODO: unwrap because the string length should not exceed u16
    frame.set_cursor_position((
        (2 + app.search.len()).try_into().unwrap(),
        frame.area().height,
    ));
}
