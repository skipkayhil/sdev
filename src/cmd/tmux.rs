use std::sync::Arc;

use jwalk::WalkDirGeneric;
use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;

use crate::dep::{tmux, Dep};
use crate::repo::GitRepo;
use crate::shell;
use crate::tui::Tui;

mod ui;

enum Status {
    Running,
    Finished(Option<GitRepo>),
}

struct App {
    config: crate::Config,
    nucleo: Nucleo<GitRepo>,
    search: String,
    selected: u32,
    state: ListState,
    status: Status,
}

impl App {
    pub fn new(config: crate::Config) -> Self {
        let nucleo = Nucleo::<GitRepo>::new(Config::DEFAULT, Arc::new(|| {}), None, 1);
        let state = ListState::default().with_selected(Some(0));

        Self {
            config,
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
            .map(|item| item.data)
            .cloned();

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

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn run(config: crate::Config) -> anyhow::Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(config);

    {
        let walk_dir = WalkDirGeneric::<((), bool)>::new(&app.config.root).process_read_dir(
            |_depth, _path, _read_dir_state, children| {
                for dir_entry in children.iter_mut().flatten() {
                    if dir_entry.path().join(".git").read_dir().is_ok() {
                        dir_entry.read_children_path = None;
                        dir_entry.client_state = true;
                    }
                }
            },
        );

        for dir_entry in walk_dir.into_iter().flatten() {
            if !dir_entry.client_state {
                continue;
            };

            if let Some(name) = dir_entry.file_name.to_str() {
                let repo = GitRepo::new(name.into(), dir_entry.path());

                app.nucleo.injector().push(repo, |repo_ref, dst| {
                    dst[0] = repo_ref.path().to_string_lossy().into()
                });
            }
        }
    };

    while app.is_running() {
        app.tick();

        tui.terminal.draw(|frame| ui::render(&mut app, frame))?;

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

    let Status::Finished(Some(repo)) = app.status else {
        return Ok(());
    };

    let name = tmux::SessionName::from(repo.name());

    tmux::Session::new(name.clone(), repo.path().to_owned()).process()?;

    let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

    shell::new!("tmux", subcommand, "-t", &name.0).run(false)?;

    Ok(())
}
