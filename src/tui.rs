use std::io::stderr;
use std::panic;

use ratatui::crossterm::{execute, terminal};

use ratatui::{backend::CrosstermBackend, Terminal};

pub type CrosstermTerminal = Terminal<CrosstermBackend<std::io::Stderr>>;

pub struct Tui {
    pub terminal: CrosstermTerminal,
}

impl Tui {
    pub fn new() -> anyhow::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(std::io::stderr(), terminal::EnterAlternateScreen,)?;

        // In case we panic, set a hook to reset the terminal
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");

            panic_hook(panic);
        }));

        self.terminal.clear()?;

        Ok(())
    }

    pub fn reset() -> anyhow::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(std::io::stderr(), terminal::LeaveAlternateScreen,)?;
        Ok(())
    }
}
