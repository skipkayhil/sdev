use crate::dep::{Dep, MeetResult, MetResult};

use std::fs;
use std::path::PathBuf;

use gix::interrupt::IS_INTERRUPTED;
use gix::progress::Discard;
use gix::Url;
use ratatui::{
    backend::CrosstermBackend,
    crossterm,
    prelude::{Line, Stylize, Widget},
    style::Color,
    widgets::Paragraph,
    Terminal, TerminalOptions, Viewport,
};

pub struct Clone {
    url: Url,
    path: PathBuf,
}

impl Clone {
    pub fn new(url: Url, path: PathBuf) -> Self {
        Clone { url, path }
    }
}

impl Dep for Clone {
    fn met(&self) -> MetResult {
        Ok(self.path.join(".git").is_dir().into())
    }

    fn meet(&self) -> MeetResult {
        crossterm::terminal::enable_raw_mode()?;

        let mut terminal = Terminal::with_options(
            CrosstermBackend::new(std::io::stdout()),
            TerminalOptions {
                viewport: Viewport::Inline(5),
            },
        )?;

        terminal.insert_before(1, |buf| {
            Line::from(vec!["src".dark_gray(), format!(" {}", &self.url).into()])
                .render(buf.area, buf);
        })?;
        terminal.insert_before(1, |buf| {
            Line::from(vec![
                "dst".dark_gray(),
                format!(" {}", &self.path.to_string_lossy()).into(),
            ])
            .render(buf.area, buf);
        })?;

        fs::create_dir_all(&self.path)?;

        let mut prepare_clone = gix::prepare_clone(self.url.clone(), &self.path)?;

        terminal.draw(|f| {
            f.render_widget("cloning...", f.area());
        })?;

        let (mut prepare_checkout, _) =
            prepare_clone.fetch_then_checkout(Discard, &IS_INTERRUPTED)?;

        terminal.insert_before(1, |buf| {
            Line::from(vec!["✓".fg(Color::Indexed(42)), " fetched".into()]).render(buf.area, buf);
        })?;

        terminal.draw(|f| {
            f.render_widget("checking out...", f.area());
        })?;

        prepare_checkout.main_worktree(Discard, &IS_INTERRUPTED)?;

        terminal.insert_before(1, |buf| {
            Paragraph::new(Line::from(vec![
                "✓".fg(Color::Indexed(42)),
                " cloned".into(),
            ]))
            .render(buf.area, buf);
        })?;

        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}
