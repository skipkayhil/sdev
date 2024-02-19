use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{CrosstermBackend, Line, Style, Stylize, Terminal},
    widgets::{Block, Borders, List, ListDirection, ListState},
};
use std::io::stdout;

use crate::repositories::git_repos::{CachingRepository, FileSystemRepository, Repository};
use crate::Config;

const CHEVRON: &str = ">";

pub fn run(config: Config) -> anyhow::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut repository = CachingRepository::new(FileSystemRepository::new(config.root));
    let all_paths = repository.fetch_all()?;

    let mut search = String::new();
    let mut state = ListState::default().with_selected(Some(0));

    let padded_chevron = format!("{CHEVRON} ");

    loop {
        terminal.draw(|frame| {
            let layout = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)])
                .split(frame.size());

            // TODO: unwrap because paths should all be utf8... but it feels like a lib may help
            // here
            let path_strings: Vec<&str> = all_paths
                .iter()
                .map(|repo| repo.path().to_str().unwrap())
                .collect();
            let path_list = List::new(path_strings)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::new().dark_gray()),
                )
                .highlight_symbol(&padded_chevron)
                .highlight_style(Style::new().bold())
                .direction(ListDirection::BottomToTop);

            frame.render_stateful_widget(path_list, layout[0], &mut state);

            let prompt = Line::from(vec![
                padded_chevron.clone().bold().magenta(),
                search.clone().bold(),
            ]);

            frame.render_widget(prompt, layout[1]);

            // TODO: unwrap because the string length should not exceed u16
            frame.set_cursor((2 + search.len()).try_into().unwrap(), frame.size().height);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char(key) => search.push(key),
                        KeyCode::Backspace => {
                            search.pop();
                        }
                        KeyCode::Up => match state.selected() {
                            None => state.select(Some(0)),
                            Some(i) if i == all_paths.len() - 1 => (),
                            Some(i) => state.select(Some(i + 1)),
                        },
                        KeyCode::Down => match state.selected() {
                            None => state.select(Some(0)),
                            Some(0) => (),
                            Some(i) => state.select(Some(i - 1)),
                        },
                        _ => (),
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
