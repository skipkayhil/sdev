use std::io;
use std::path::Path;

use jwalk::WalkDirGeneric;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::ListState};

use crate::dep::{Dep, tmux};
use crate::repo::GitRepo;
use crate::shell;
use crate::ui::picker::PickerState;

mod ui;

enum Status {
    Running,
    Finished(Option<GitRepo>),
}

struct App {
    repo_picker: PickerState,
    search: String,
    selected: u32,
    state: ListState,
    status: Status,
}

impl App {
    pub fn new() -> Self {
        let state = ListState::default().with_selected(Some(0));

        Self {
            repo_picker: PickerState::default(),
            search: String::new(),
            selected: 0,
            state,
            status: Status::Running,
        }
    }

    pub fn pop_char(&mut self) {
        self.search.pop();
        self.repo_picker.pop_char(&self.search);
    }

    pub fn push_char(&mut self, c: char) {
        self.search.push(c);
        self.repo_picker.push_char(&self.search);
    }

    pub fn dec_selection(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        // TODO: unwrap because List uses usize, custom List will fix that
        self.state.select(Some(self.selected.try_into().unwrap()));
    }

    pub fn inc_selection(&mut self) {
        let incremented_selection = self.selected.saturating_add(1);

        if self.repo_picker.matched_item_count() > incremented_selection {
            self.selected = self.selected.saturating_add(1);
            // TODO: unwrap because List uses usize, custom List will fix that
            self.state.select(Some(self.selected.try_into().unwrap()));
        }
    }

    pub fn abort(&mut self) {
        self.status = Status::Finished(None)
    }

    pub fn complete(&mut self) {
        let selected_string = self.repo_picker.get_matched_item(self.selected).cloned();

        self.status = Status::Finished(selected_string)
    }

    pub fn is_running(&self) -> bool {
        matches!(&self.status, Status::Running)
    }

    pub fn tick(&mut self) {
        let status = &self.repo_picker.tick();

        if status.changed && self.repo_picker.matched_item_count() <= self.selected {
            self.selected = self.repo_picker.matched_item_count().saturating_sub(1);
            // TODO: unwrap because List uses usize, custom List will fix that
            self.state.select(Some(self.selected.try_into().unwrap()));
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, root: &Path) -> io::Result<()> {
        {
            let walk_dir = WalkDirGeneric::<((), bool)>::new(root).process_read_dir(
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

                    self.repo_picker.push(repo, root);
                }
            }
        };

        while self.is_running() {
            self.tick();

            terminal.draw(|frame| ui::render(self, frame))?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Esc => self.abort(),
                            KeyCode::Char(key) => self.push_char(key),
                            KeyCode::Backspace => self.pop_char(),
                            KeyCode::Up => self.inc_selection(),
                            KeyCode::Down => self.dec_selection(),
                            KeyCode::Enter => self.complete(),
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

const CMD_ATTACH: &str = "attach-session";
const CMD_SWITCH: &str = "switch-client";

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn run(config: crate::Config) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    let app_result = app.run(&mut terminal, &config.root);

    ratatui::restore();

    // Ensure the terminal is reset before possibly returning early
    app_result?;

    let Status::Finished(Some(repo)) = app.status else {
        return Ok(());
    };

    let name = tmux::SessionName::from(repo.name());

    tmux::Session::new(name.clone(), repo.path().to_owned()).process()?;

    let subcommand = if in_tmux() { CMD_SWITCH } else { CMD_ATTACH };

    shell::new!("tmux", subcommand, "-t", &name.0).run(false)?;

    Ok(())
}
