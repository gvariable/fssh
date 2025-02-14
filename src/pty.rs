use std::{
    io::Write,
    sync::{atomic::AtomicBool, Mutex},
    thread::sleep,
    time::Duration,
};

use anyhow::Ok;
pub use portable_pty::CommandBuilder;
use portable_pty::{native_pty_system, MasterPty, PtySize};
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::spawn_blocking,
};
use tui_term::{vt100::Parser, widget::Cursor, widget::PseudoTerminal as PseudoTerminalWidget};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Block,
};

use crate::Terminal;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    rows: u16,
    cols: u16,
}

impl Size {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self { rows, cols }
    }
}

/// A pseudo terminal that spawns an SSH client and captures the password.
pub struct PseudoTerminal {
    parser: Arc<RwLock<Parser>>,
    sender: Sender<Bytes>,
    master: Box<dyn MasterPty>,
    terminate: Arc<AtomicBool>,
    buffer: Arc<Mutex<String>>,
}

impl PseudoTerminal {
    /// Creates a [`PseudoTerminal`] instance.
    ///
    /// This function opens a pseudo terminal (pty), spawns an SSH client in the slave pty, and
    /// manages communication between the master and slave pty. The master pty reads output from
    /// the slave pty and writes input to the slave pty. It attempts to connect to the SSH server
    /// using the provided password (if applicable). If the connection is successful, it sends the
    /// output for rendering; otherwise, it prompts the user to input the password and memorizes it.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the terminal (rows and columns).
    /// * `cmd` - The command to be executed in the pseudo terminal.
    /// * `passwd` - An optional password that may be used by the SSH client.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the newly created [`PseudoTerminal`] instance,
    /// or an error if the creation failed.
    pub fn new(
        size: Size,
        cmd: CommandBuilder,
        mut passwd: Option<String>,
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let parser = Arc::new(RwLock::new(Parser::new(size.rows, size.cols, 1000)));

        let terminate = Arc::new(AtomicBool::new(false));
        {
            // Spawning a new thread to run the command
            spawn_blocking(move || -> anyhow::Result<()> {
                let mut child = pty_pair.slave.spawn_command(cmd)?;
                let _ = child.wait();
                drop(pty_pair.slave);
                Ok(())
            });
        }

        let buffer = Arc::new(Mutex::new(String::new()));

        let (tx, mut rx) = channel::<Bytes>(32);

        // pty reader end
        {
            let mut reader = pty_pair.master.try_clone_reader().unwrap();
            let parser = parser.clone();
            let terminate = terminate.clone();
            let buffer = buffer.clone();
            let tx = tx.clone();

            spawn_blocking(move || -> anyhow::Result<()> {
                let mut buf = [0; 1024];
                let mut send_passwd = false;
                let mut validate_passwd = false;

                loop {
                    let n = reader.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }

                    let string = String::from_utf8_lossy(&buf[..n]);

                    if passwd.is_some() && string.contains("assword: ") {
                        // the unwrap is safe here because we have already checked
                        let passwd = passwd.take().unwrap();
                        tx.blocking_send(Bytes::from(format!("{}\n", passwd)))?;
                        send_passwd = true;
                    } else {
                        if send_passwd && !validate_passwd {
                            // skip the whitespaces bytes
                            if string.trim().is_empty() {
                                continue;
                            }

                            validate_passwd = true;
                            let mut parser = parser.write().map_err(|_| {
                                anyhow::anyhow!("Failed to acquire write lock of Parser.")
                            })?;
                            if string.contains("ermission denied") {
                                parser.process(&Bytes::from(format!(
                                    "\x1b[1;4mCached password is outdated, please input it again.\x1b[0m\n"
                                )));
                            } else {
                                parser.process(&buf[..n]);
                            }
                        } else {
                            let mut parser = parser.write().map_err(|_| {
                                anyhow::anyhow!("Failed to acquire write lock of Parser.")
                            })?;
                            parser.process(&buf[..n]);
                        }
                    }

                    let mut buffer = buffer
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Failed to acquire lock of buffer."))?;
                    if buffer.len() < 1024 {
                        buffer.push_str(&string);
                    }
                }

                // wait for a while before rendering the remaining data
                sleep(Duration::from_millis(10));
                terminate.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            });
        }

        {
            let mut writer = pty_pair.master.take_writer().unwrap();
            let buffer = buffer.clone();
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    writer.write_all(&data)?;
                    writer.flush()?;

                    let mut buffer = buffer
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Failed to acquire lock of buffer."))?;

                    if buffer.len() < 4096 {
                        buffer.push_str(String::from_utf8_lossy(&data).as_ref());
                    }
                }
                Ok(())
            });
        }

        Ok(Self {
            parser: parser,
            sender: tx,
            master: pty_pair.master,
            terminate,
            buffer: buffer,
        })
    }

    async fn handle_key_event(&mut self, key: &KeyEvent) -> anyhow::Result<bool> {
        let input_bytes = match key.code {
            KeyCode::Char(ch) => {
                let mut send = ch.to_string().into_bytes();

                if key.modifiers == KeyModifiers::CONTROL {
                    let upper = ch.to_ascii_uppercase();
                    match upper {
                        // https://github.com/fyne-io/terminal/blob/master/input.go
                        // https://gist.github.com/ConnerWill/d4b6c776b509add763e17f9f113fd25b
                        '2' | '@' | ' ' => send = vec![0],
                        '3' | '[' => send = vec![27],
                        '4' | '\\' => send = vec![28],
                        '5' | ']' => send = vec![29],
                        '6' | '^' => send = vec![30],
                        '7' | '-' | '_' => send = vec![31],
                        char if ('A'..='_').contains(&char) => {
                            // Since A == 65,
                            // we can safely subtract 64 to get
                            // the corresponding control character
                            let ascii_val = char as u8;
                            let ascii_to_send = ascii_val - 64;
                            send = vec![ascii_to_send];
                        }
                        _ => {}
                    }
                }
                send
            }
            #[cfg(unix)]
            KeyCode::Enter => vec![b'\n'],
            #[cfg(windows)]
            KeyCode::Enter => vec![b'\r', b'\n'],
            KeyCode::Backspace => vec![8],
            KeyCode::Left => vec![27, 91, 68],
            KeyCode::Right => vec![27, 91, 67],
            KeyCode::Up => vec![27, 91, 65],
            KeyCode::Down => vec![27, 91, 66],
            KeyCode::Tab => vec![9],
            KeyCode::Home => vec![27, 91, 72],
            KeyCode::End => vec![27, 91, 70],
            KeyCode::PageUp => vec![27, 91, 53, 126],
            KeyCode::PageDown => vec![27, 91, 54, 126],
            KeyCode::BackTab => vec![27, 91, 90],
            KeyCode::Delete => vec![27, 91, 51, 126],
            KeyCode::Insert => vec![27, 91, 50, 126],
            KeyCode::Esc => vec![27],
            _ => return Ok(true),
        };

        self.sender.send(Bytes::from(input_bytes)).await?;
        Ok(true)
    }

    /// Renders the output from the slave pty, processes input from the keyboard, and searches for the password when the pty exits.
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<impl Write>,
    ) -> anyhow::Result<Option<String>> {
        let terminal_size = terminal.size()?;
        let mut size = Size {
            rows: terminal_size.height,
            cols: terminal_size.width,
        };

        loop {
            if self.terminate.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            terminal.draw(|frame| {
                let parser = self
                    .parser
                    .read()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire read lock of Parser."))
                    .unwrap();

                let screen = parser.screen();
                let block = Block::default().style(Style::default().bg(Color::Black));

                let cursor = Cursor::default().visibility(true);
                // Style
                let pseudo_term = PseudoTerminalWidget::new(screen)
                    .block(block)
                    .cursor(cursor);

                let rect = Rect::new(0, 0, size.cols, size.rows);
                frame.render_widget(pseudo_term, rect);
            })?;

            if event::poll(Duration::from_millis(10))? {
                match event::read()? {
                    Event::FocusLost => {}
                    Event::Key(key) => {
                        self.handle_key_event(&key).await?;
                    }
                    Event::Resize(cols, rows) => {
                        size.rows = rows;
                        size.cols = cols;
                        self.parser
                            .write()
                            .map_err(|_| {
                                anyhow::anyhow!("Failed to acquire write lock of Parser.")
                            })?
                            .set_size(rows, cols);

                        self.master
                            .resize(PtySize {
                                rows: rows,
                                cols: cols,
                                pixel_width: 0,
                                pixel_height: 0,
                            })
                            .unwrap();
                    }

                    _ => {}
                }
            }
        }

        let mut passwd: Option<String> = None;

        let buffer = self.buffer.lock().unwrap();
        if buffer.contains("Last login") {
            if let Some(start) = buffer.rfind("password: ") {
                let end = buffer[start + 10..].find('\n').unwrap();
                passwd = Some(
                    String::from(&buffer[start + 10..start + 10 + end])
                        .trim()
                        .to_string(),
                );
            }
        }

        Ok(passwd)
    }
}
