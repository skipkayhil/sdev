use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Line, Style, Stylize},
    widgets::{Block, Borders, List, ListDirection},
    Frame,
};

use crate::cmd::tmux::App;

const PADDED_CHEVRON: &str = "> ";

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout =
        Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(frame.area());

    let snap = app.nucleo.snapshot();
    let matched_paths: Vec<String> = snap
        .matched_items(0..snap.matched_item_count().min(layout[0].height.into()))
        .map(|item| {
            item.data
                .relative_path(&app.config.root)
                .to_string_lossy()
                .into()
        })
        .collect();

    let path_list = List::new(matched_paths)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::new().dark_gray()),
        )
        .highlight_symbol(PADDED_CHEVRON)
        .highlight_style(Style::new().bold().white())
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
