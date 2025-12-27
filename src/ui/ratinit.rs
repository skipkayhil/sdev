use std::boxed::Box;

// The MIT License (MIT)
//
// Copyright (c) 2016-2022 Florian Dehau
// Copyright (c) 2023-2025 The Ratatui Developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::io::{self, Stdout, stdout};

use ratatui_core::terminal::{Terminal, TerminalOptions};
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::execute;
use ratatui_crossterm::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> DefaultTerminal {
    try_init().expect("failed to initialize terminal")
}

pub fn try_init() -> io::Result<DefaultTerminal> {
    set_panic_hook();
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

pub fn init_with_options(options: TerminalOptions) -> DefaultTerminal {
    try_init_with_options(options).expect("failed to initialize terminal")
}

pub fn try_init_with_options(options: TerminalOptions) -> io::Result<DefaultTerminal> {
    set_panic_hook();
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::with_options(backend, options)
}

pub fn restore() {
    if let Err(err) = try_restore() {
        std::eprintln!("Failed to restore terminal: {err}");
    }
}

pub fn try_restore() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();

    // hartley: replaced alloc with std since I'm not using no-std
    std::panic::set_hook(Box::new(move |info| {
        restore();
        hook(info);
    }));
}
