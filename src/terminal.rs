use std::{
    io::{stdout, Stdout, Write},
    ops::{Deref, DerefMut},
};

use color_eyre::{config::HookBuilder, eyre::Ok};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::{self, backend::CrosstermBackend, TerminalOptions, Viewport};

type TerminalBackend<W> = ratatui::Terminal<CrosstermBackend<W>>;

pub struct Terminal<W: Write> {
    inner: TerminalBackend<W>,
}

impl Terminal<Stdout> {
    pub fn new(height: u16) -> color_eyre::Result<Self> {
        enable_raw_mode()?;

        init_error_hooks()?;
        let backend = CrosstermBackend::new(stdout());
        let options = TerminalOptions {
            viewport: Viewport::Inline(height),
        };
        let terminal = ratatui::Terminal::with_options(backend, options)?;
        Ok(Self { inner: terminal })
    }
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));

    Ok(())
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
    disable_raw_mode()?;
    Ok(())
}
