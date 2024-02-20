use std::marker::{Send, Sync};
use std::sync::Arc;

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

use crate::repositories::git_repos::{CachingRepository, FileSystemRepository, Repository};
use crate::tui::Tui;

const CHEVRON: &str = ">";

enum Status {
    Running,
    Finished(Option<String>),
}

struct App<T: Send + Sync + 'static> {
    nucleo: Nucleo<T>,
    search: String,
    selected: u32,
    state: ListState,
    status: Status,
}

impl<T: Send + Sync + 'static> App<T> {
    pub fn new() -> Self {
        let nucleo = Nucleo::<T>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let state = ListState::default().with_selected(Some(0));

        Self {
            nucleo,
            search: String::new(),
            selected: 0,
            state,
            status: Status::Running,
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

    pub fn abort(&mut self) {
        self.status = Status::Finished(None)
    }

    pub fn complete(&mut self) {
        let selected_string = self
            .nucleo
            .snapshot()
            .get_matched_item(self.selected)
            .map(|item| item.matcher_columns[0].to_string());

        self.status = Status::Finished(selected_string)
    }

    pub fn is_running(&self) -> bool {
        matches!(&self.status, Status::Running)
    }

    pub fn tick(&mut self) {
        let status = self.nucleo.tick(10);

        if status.changed && self.nucleo.snapshot().matched_item_count() <= self.selected {
            self.selected = self
                .nucleo
                .snapshot()
                .matched_item_count()
                .saturating_sub(1);
            // TODO: unwrap because List uses usize, custom List will fix that
            self.state.select(Some(self.selected.try_into().unwrap()));
        }
    }
}

pub fn run(config: crate::Config) -> anyhow::Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new();

    let injector = app.nucleo.injector();

    let mut repository = CachingRepository::new(FileSystemRepository::new(config.root));
    let all_repos = repository.fetch_all()?;

    for repo in all_repos.iter() {
        injector.push(repo.path().to_owned(), |dst| {
            dst[0] = repo.path().to_string_lossy().into()
        });
    }

    let padded_chevron = format!("{CHEVRON} ");

    while app.is_running() {
        app.tick();

        tui.terminal.draw(|frame| {
            let layout = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)])
                .split(frame.size());

            let snap = app.nucleo.snapshot();
            let matched_paths: Vec<String> = snap
                .matched_items(0..snap.matched_item_count().min(layout[0].height.into()))
                .map(|item| item.data.to_string_lossy().into())
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
                        KeyCode::Esc => app.abort(),
                        KeyCode::Char(key) => app.push_char(key),
                        KeyCode::Backspace => app.pop_char(),
                        KeyCode::Up => app.inc_selection(),
                        KeyCode::Down => app.dec_selection(),
                        KeyCode::Enter => app.complete(),
                        _ => (),
                    }
                }
            }
        }
    }

    Tui::reset()?;

    if let Status::Finished(Some(selected)) = app.status {
        println!("{selected}");
    }

    Ok(())
}
