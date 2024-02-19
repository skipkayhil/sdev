use crossterm::event::{self, KeyCode, KeyEventKind};
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Line, Style, Stylize},
    widgets::{Block, Borders, List, ListDirection, ListState},
};
use std::sync::Arc;

use crate::repositories::git_repos::{CachingRepository, FileSystemRepository, Repository};
use crate::tui::Tui;

const CHEVRON: &str = ">";

struct App {
    nucleo: Nucleo<usize>,
    search: String,
    state: ListState,
}

impl App {
    pub fn new() -> Self {
        let nucleo = Nucleo::<usize>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let state = ListState::default().with_selected(Some(0));

        Self {
            nucleo,
            search: String::new(),
            state,
        }
    }

    pub fn pop_char(&mut self) {
        self.search.pop();
        self.nucleo.pattern.reparse(
            0,
            &self.search,
            CaseMatching::Smart,
            Normalization::Smart,
            false,
        );
    }

    pub fn push_char(&mut self, c: char) {
        self.search.push(c);
        self.nucleo.pattern.reparse(
            0,
            &self.search,
            CaseMatching::Smart,
            Normalization::Smart,
            true,
        );
    }

    pub fn tick(&mut self) {
        self.nucleo.tick(10);
    }
}

pub fn run(config: crate::Config) -> anyhow::Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new();

    let injector = app.nucleo.injector();

    let mut repository = CachingRepository::new(FileSystemRepository::new(config.root));
    let all_paths = repository.fetch_all()?;
    // TODO: unwrap because paths should all be utf8... but it feels like a lib may help
    // here
    let path_strings: Vec<&str> = all_paths
        .iter()
        .map(|repo| repo.path().to_str().unwrap())
        .collect();

    for (i, path) in path_strings.iter().enumerate() {
        injector.push(i, |dst| dst[0] = path.to_string().into());
    }

    let padded_chevron = format!("{CHEVRON} ");

    loop {
        app.tick();

        tui.terminal.draw(|frame| {
            let layout = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)])
                .split(frame.size());

            let snap = app.nucleo.snapshot();
            let matched_paths: Vec<&str> = snap
                .matched_items(0..snap.matched_item_count().min(layout[0].height.into()))
                .map(|item| path_strings[*item.data])
                .collect();

            let path_list = List::new(matched_paths)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::new().dark_gray()),
                )
                .highlight_symbol(&padded_chevron)
                .highlight_style(Style::new().bold())
                .direction(ListDirection::BottomToTop);

            frame.render_stateful_widget(path_list, layout[0], &mut app.state);

            let prompt = Line::from(vec![
                padded_chevron.clone().bold().magenta(),
                app.search.clone().bold(),
            ]);

            frame.render_widget(prompt, layout[1]);

            // TODO: unwrap because the string length should not exceed u16
            frame.set_cursor(
                (2 + app.search.len()).try_into().unwrap(),
                frame.size().height,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char(key) => app.push_char(key),
                        KeyCode::Backspace => app.pop_char(),
                        KeyCode::Up => match app.state.selected() {
                            None => app.state.select(Some(0)),
                            Some(i) if i == all_paths.len() - 1 => (),
                            Some(i) => app.state.select(Some(i + 1)),
                        },
                        KeyCode::Down => match app.state.selected() {
                            None => app.state.select(Some(0)),
                            Some(0) => (),
                            Some(i) => app.state.select(Some(i - 1)),
                        },
                        _ => (),
                    }
                }
            }
        }
    }

    Tui::reset()
}
