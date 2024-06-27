use std::{
    error::Error,
    io::{stdout, Stdout, Write},
    ops::{Deref, DerefMut},
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{self, backend::CrosstermBackend};

type TerminalBackend<W> = ratatui::Terminal<CrosstermBackend<W>>;

pub struct Terminal<W: Write> {
    inner: TerminalBackend<W>,
    alternate_screen: bool,
}

impl Terminal<Stdout> {
    pub fn new(height: Option<u16>, alternate_screen: bool) -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();

        if alternate_screen {
            execute!(stdout, EnterAlternateScreen)?;
        }

        let backend = CrosstermBackend::new(stdout);
        let terminal = if let Some(height) = height {
            let options = ratatui::TerminalOptions {
                viewport: ratatui::Viewport::Inline(height),
            };
            ratatui::Terminal::with_options(backend, options)?
        } else {
            ratatui::Terminal::new(backend)?
        };

        Result::Ok(Self {
            inner: terminal,
            alternate_screen,
        })
    }
}

impl<W: Write> Deref for Terminal<W> {
    type Target = TerminalBackend<W>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W: Write> DerefMut for Terminal<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<W: Write> Drop for Terminal<W> {
    fn drop(&mut self) {
        if let Err(err) = restore_terminal(self.alternate_screen) {
            eprintln!("Failed to restore terminal: {}", err);
        }
    }
}

fn restore_terminal(alternate_screen: bool) -> Result<(), Box<dyn Error>> {
    if alternate_screen {
        execute!(stdout(), LeaveAlternateScreen)?;
    }
    disable_raw_mode()?;
    Ok(())
}
