use std::fs;
use std::path::PathBuf;

use gix::interrupt::IS_INTERRUPTED;
use gix::progress::Discard;
use gix::Url;
use ratatui::{
    prelude::{Line, Stylize, Widget},
    widgets::Paragraph,
    DefaultTerminal, TerminalOptions, Viewport,
};

use crate::dep::{Dep, MeetResult, MetResult};

pub struct Clone {
    url: Url,
    path: PathBuf,
}

impl Clone {
    pub fn new(url: Url, path: PathBuf) -> Self {
        Clone { url, path }
    }

    fn run(&self, terminal: &mut DefaultTerminal) -> MeetResult {
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
            Line::from(vec!["✓".green(), " fetched".into()]).render(buf.area, buf);
        })?;

        terminal.draw(|f| {
            f.render_widget("checking out...", f.area());
        })?;

        prepare_checkout.main_worktree(Discard, &IS_INTERRUPTED)?;

        terminal.insert_before(1, |buf| {
            Paragraph::new(Line::from(vec!["✓".green(), " cloned".into()])).render(buf.area, buf);
        })?;

        Ok(())
    }
}

impl Dep for Clone {
    fn met(&self) -> MetResult {
        Ok(self.path.join(".git").is_dir().into())
    }

    fn meet(&self) -> MeetResult {
        let mut terminal = ratatui::init_with_options(TerminalOptions {
            viewport: Viewport::Inline(5),
        });

        let result = self.run(&mut terminal);

        ratatui::restore();

        result
    }
}
