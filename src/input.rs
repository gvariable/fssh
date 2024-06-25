use crossterm::event::Event;
use std::ops::{Deref, DerefMut};
use unicode_width::UnicodeWidthStr;

use tui_input::{backend::crossterm::EventHandler, Input};

pub(crate) struct InputBuffer {
    pub(crate) input: Input,
    pub(crate) prompt: String,
}

impl Deref for InputBuffer {
    type Target = Input;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl DerefMut for InputBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl InputBuffer {
    pub fn new(prompt: String) -> Self {
        Self {
            input: Input::default(),
            prompt,
        }
    }

    pub(crate) fn value(&self) -> String {
        self.prompt.clone() + self.input.value()
    }

    pub(crate) fn visual_cursor(&self) -> usize {
        UnicodeWidthStr::width(self.prompt.as_str()) + self.input.visual_cursor()
    }

    pub(crate) fn handle_event(&mut self, event: Event) {
        self.input.handle_event(&event);
    }
}
