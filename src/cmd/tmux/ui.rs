use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    prelude::{Line, Stylize},
};

use crate::cmd::tmux::App;

const PADDED_CHEVRON: &str = "> ";

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout =
        Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(frame.area());

    let prompt = Line::from(vec![
        PADDED_CHEVRON.bold().magenta(),
        app.search.clone().bold(),
    ]);

    app.repo_picker.render(layout[0], frame.buffer_mut());

    frame.render_widget(prompt, layout[1]);

    // TODO: unwrap because the string length should not exceed u16
    frame.set_cursor_position((
        (2 + app.search.len()).try_into().unwrap(),
        frame.area().height,
    ));
}
