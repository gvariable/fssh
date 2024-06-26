use std::{
    error::Error,
    io::{stdout, Stdout, Write},
    ops::{Deref, DerefMut},
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen},
};

use ratatui::{self, backend::CrosstermBackend};

type TerminalBackend<W> = ratatui::Terminal<CrosstermBackend<W>>;

pub struct Terminal<W: Write> {
    inner: TerminalBackend<W>,
}

impl Terminal<Stdout> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Clear(crossterm::terminal::ClearType::All)
        )?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        Result::Ok(Self { inner: terminal })
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
        let _ = restore_terminal();
    }
}

fn restore_terminal() -> color_eyre::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
