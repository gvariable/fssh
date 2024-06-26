use std::io::Write;
use std::{error::Error, io, process::Command};

use crate::input::InputBuffer;
use crate::sshconfig::SshConfigItem;
use crate::terminal::Terminal;

use ratatui::prelude::*;
use ratatui::widgets::*;

use crossterm::event::{self, Event};
use unicode_width::UnicodeWidthStr;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

const INFO_TEXT_NORMAL_MODE: &str =
    "(Esc) quit | (↑) move up | (↓) move down | (Enter) connect | (/) search";
const INFO_TEXT_SEAERCH_MODE: &str =
    "(Esc) quit search | (↑) move up | (↓) move down | (Enter) connect";
const SEARCH_SYMBOL: &str = "🔍 ";

enum Mode {
    Normal,
    Search,
}

pub struct SelectBox {
    pub data: Vec<SshConfigItem>,
    state: TableState,
    longest_item_lens: (u16, u16, u16), // order is (host, user, hostname)
    displayed_rows: usize,
    input_buffer: InputBuffer,
    mode: Mode,
}

impl SelectBox {
    pub fn new(data: Vec<SshConfigItem>) -> Self {
        Self {
            longest_item_lens: (
                data.iter()
                    .map(|d| UnicodeWidthStr::width(d.host.as_str()))
                    .max()
                    .unwrap_or(0) as u16,
                data.iter()
                    .map(|d| UnicodeWidthStr::width(d.user.as_str()))
                    .max()
                    .unwrap_or(0) as u16,
                data.iter()
                    .map(|d| UnicodeWidthStr::width(d.hostname.as_str()))
                    .max()
                    .unwrap_or(0) as u16,
            ),
            displayed_rows: data.len(),
            state: TableState::default().with_selected(0),
            input_buffer: InputBuffer::new(SEARCH_SYMBOL.to_string()),
            mode: Mode::Normal,
            data,
        }
    }

    pub fn select(
        &mut self,
        terminal: &mut Terminal<impl Write>,
    ) -> io::Result<Option<SshConfigItem>> {
        let mut selected: Option<SshConfigItem> = None;
        loop {
            self.draw(terminal)?;
            if let Event::Key(key) = event::read()? {
                use event::KeyCode::*;
                match key.code {
                    Down => self.down(),
                    Up => self.up(),
                    Enter => {
                        // If no host is selected, do nothing
                        if self.state.selected().is_none() {
                            continue;
                        } else {
                            selected = self.data.get(self.state.selected().unwrap()).cloned();
                            // clear the current buffer
                            terminal.clear()?;
                            break;
                        }
                    }
                    _ => {
                        if matches!(self.mode, Mode::Normal) {
                            match key.code {
                                Esc => {
                                    terminal.clear()?;
                                    break;
                                }
                                Char('/') => {
                                    self.mode = Mode::Search;
                                    self.input_buffer.reset();
                                }

                                _ => {}
                            }
                        } else {
                            match key.code {
                                Esc => {
                                    self.input_buffer.reset();
                                    self.mode = Mode::Normal;
                                }
                                _ => {
                                    self.input_buffer.handle_event(Event::Key(key));
                                }
                            }
                        }
                    }
                }
            }
        }
        Result::Ok(selected)
    }

    pub fn draw(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        terminal.draw(|frame| {
            self.ui(frame);
        })?;
        Result::Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let header = Row::new(vec![
            Cell::from("Host").style(Style::default().add_modifier(Modifier::UNDERLINED)),
            Cell::from("User").style(Style::default().add_modifier(Modifier::UNDERLINED)),
            Cell::from("Hostname").style(Style::default().add_modifier(Modifier::UNDERLINED)),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        // if the input buffer is empty, show all the data
        let rows: Vec<_> = if self.input_buffer.input.value().is_empty() {
            self.data
                .iter()
                .map(|d| {
                    Row::new(vec![
                        Cell::from(d.host.as_str()),
                        Cell::from(d.user.as_str()),
                        Cell::from(d.hostname.as_str()),
                    ])
                })
                .collect()
        } else {
            // if the input buffer is not empty, show the filtered and highlighted data
            let matches = self.fuzzy_match();
            matches
                .iter()
                .map(|(config, indices)| {
                    let host = Text::from(Line::from(Self::get_highlight_spans(
                        &config.host,
                        &indices[0],
                    )));
                    let user = Text::from(Line::from(Self::get_highlight_spans(
                        &config.user,
                        &indices[1],
                    )));
                    let hostname = Text::from(Line::from(Self::get_highlight_spans(
                        &config.hostname,
                        &indices[2],
                    )));
                    Row::new([host, user, hostname])
                })
                .collect()
        };

        self.displayed_rows = rows.len();

        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2),
            ],
        )
        .header(header)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_spacing(HighlightSpacing::Always);

        let info = if matches!(self.mode, Mode::Normal) {
            Paragraph::new(Line::from(INFO_TEXT_NORMAL_MODE)).centered()
        } else {
            Paragraph::new(Line::from(INFO_TEXT_SEAERCH_MODE)).centered()
        };

        if matches!(self.mode, Mode::Search) {
            let recs = Layout::vertical([
                Constraint::Length(self.data.len() as u16 + 2),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(f.size());

            let input = Paragraph::new(
                Text::from(self.input_buffer.value()).style(Style::default().fg(Color::Cyan)),
            )
            .block(Block::default().borders(Borders::ALL));

            StatefulWidget::render(table, recs[0], f.buffer_mut(), &mut self.state);
            input.render(recs[1], f.buffer_mut());
            info.render(recs[2], f.buffer_mut());

            f.set_cursor(
                recs[1].x + 1 + self.input_buffer.visual_cursor() as u16,
                recs[1].y + 1,
            );
        } else {
            let recs = Layout::vertical([
                Constraint::Length(self.data.len() as u16 + 2),
                Constraint::Length(1),
            ])
            .split(f.size());

            StatefulWidget::render(table, recs[0], f.buffer_mut(), &mut self.state);
            info.render(recs[1], f.buffer_mut());
        }
    }

    fn up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.displayed_rows - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.displayed_rows - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    // return order: host, user, hostname
    fn fuzzy_match(&self) -> Vec<(&SshConfigItem, [Vec<usize>; 3])> {
        let matcher = SkimMatcherV2::default();
        let pattern = self.input_buffer.input.value();
        let choices: Vec<(&SshConfigItem, [Vec<usize>; 3])> = self
            .data
            .iter()
            .filter_map(|config| {
                let mut host_matched_indices = Vec::new();
                let mut user_matched_indices = Vec::new();
                let mut hostname_matched_indices = Vec::new();

                if let Some((_, indices)) = matcher.fuzzy_indices(&config.host, pattern) {
                    host_matched_indices = indices;
                }

                if let Some((_, indices)) = matcher.fuzzy_indices(&config.user, pattern) {
                    user_matched_indices = indices;
                }

                if let Some((_, indices)) = matcher.fuzzy_indices(&config.hostname, pattern) {
                    hostname_matched_indices = indices;
                }

                if host_matched_indices.is_empty()
                    && user_matched_indices.is_empty()
                    && hostname_matched_indices.is_empty()
                {
                    return None;
                } else {
                    return Some((
                        config,
                        [
                            host_matched_indices,
                            user_matched_indices,
                            hostname_matched_indices,
                        ],
                    ));
                }
            })
            .collect();
        choices
    }

    fn get_highlight_spans<'b>(input: &str, indices: &[usize]) -> Vec<Span<'b>> {
        let mut spans = Vec::new();
        let mut current_segment = String::new();
        let mut index_set: Vec<usize> = indices.to_vec();
        index_set.sort_unstable();
        index_set.dedup();

        let highlight_style = Style::default()
            .fg(Color::Rgb(250, 0, 0))
            .bg(Color::Rgb(0xFF, 0xFC, 0x67))
            .add_modifier(Modifier::BOLD);
        for (i, c) in input.chars().enumerate() {
            if index_set.contains(&i) {
                if !current_segment.is_empty() {
                    spans.push(Span::raw(current_segment.clone()));
                    current_segment.clear();
                }
                spans.push(Span::styled(c.to_string(), highlight_style));
            } else {
                current_segment.push(c);
            }
        }

        if !current_segment.is_empty() {
            spans.push(Span::raw(current_segment));
        }

        spans
    }

    pub fn connect(
        &self,
        hint: &SshConfigItem,
        passwd: Option<String>,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let passwd = String::from("password");
        Command::new("ssh")
            .arg(format!("{}@{}", hint.user, hint.host))
            .spawn()?
            .wait()?;

        Result::Ok(Some(passwd))
    }
}
