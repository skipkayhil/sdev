use std::path::{Path, PathBuf};

use clap::ValueEnum;
use jwalk::WalkDirGeneric;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};

use crate::dep::tmux::Session;
use crate::repo::GitRepo;
use crate::shell::tmux;
use crate::ui::picker::Picker;

mod ui;

#[derive(Clone, ValueEnum)]
pub enum Mode {
    Repos,
    Sessions,
}

enum Status {
    Running,
    Aborted,
    Complete,
}

struct App {
    mode: Mode,
    repo_picker: Picker<GitRepo, PathBuf>,
    session_picker: Picker<String, ()>,
    search: String,
    status: Status,
}

impl App {
    pub fn new(mode: Mode, root: &Path) -> Self {
        Self {
            mode,
            repo_picker: Picker::new(
                |repo_ref, data| repo_ref.relative_path(data).to_string_lossy().into(),
                root.to_owned(),
            ),
            session_picker: Picker::new(|str, _| str.to_string().into(), ()),
            search: String::new(),
            status: Status::Running,
        }
    }

    pub fn pop_char(&mut self) {
        self.search.pop();
        match &self.mode {
            Mode::Repos => &self.repo_picker.pop_char(&self.search),
            Mode::Sessions => &self.session_picker.pop_char(&self.search),
        };
    }

    pub fn push_char(&mut self, c: char) {
        self.search.push(c);
        match &self.mode {
            Mode::Repos => &self.repo_picker.push_char(&self.search),
            Mode::Sessions => &self.session_picker.push_char(&self.search),
        };
    }

    fn dec_selection(&mut self) {
        match &self.mode {
            Mode::Repos => self.repo_picker.dec_selection(),
            Mode::Sessions => self.session_picker.dec_selection(),
        };
    }

    fn inc_selection(&mut self) {
        match &self.mode {
            Mode::Repos => self.repo_picker.inc_selection(),
            Mode::Sessions => self.session_picker.inc_selection(),
        };
    }

    pub fn abort(&mut self) {
        self.status = Status::Aborted
    }

    pub fn complete(&mut self) {
        self.status = Status::Complete
    }

    pub fn is_running(&self) -> bool {
        matches!(&self.status, Status::Running)
    }

    fn toggle_mode(&mut self) {
        match self.mode {
            Mode::Repos => {
                self.mode = Mode::Sessions;
                self.session_picker.pop_char(&self.search);
                self.session_picker.tick();
                self.session_picker.select(self.repo_picker.selected());
            }
            Mode::Sessions => {
                self.mode = Mode::Repos;
                self.repo_picker.pop_char(&self.search);
                self.repo_picker.tick();
                self.repo_picker.select(self.session_picker.selected());
            }
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, root: &Path) -> anyhow::Result<()> {
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

                    self.repo_picker.push(repo);
                }
            }
        };

        for session in tmux::list_sessions()? {
            self.session_picker.push(session);
        }

        while self.is_running() {
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
                            KeyCode::Tab => self.toggle_mode(),
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn run(mode: &Mode, config: crate::Config) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::new(mode.clone(), &config.root);
    let app_result = app.run(&mut terminal, &config.root);

    ratatui::restore();

    // Ensure the terminal is reset before possibly returning early
    app_result?;

    if let Status::Aborted = app.status {
        return Ok(());
    };

    match &app.mode {
        Mode::Repos => {
            let Some(repo) = app.repo_picker.selected_data() else {
                return Ok(());
            };
            Session::process(repo.name().to_string(), repo.path().to_owned())?;
            Ok(tmux::attach_or_switch(repo.name())?)
        }
        Mode::Sessions => {
            let Some(name) = app.session_picker.selected_data() else {
                return Ok(());
            };
            Ok(tmux::attach_or_switch(&*name)?)
        }
    }
}
