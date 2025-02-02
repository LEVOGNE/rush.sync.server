// ## FILE: ./src/message.rs
// src/output/message.rs
use crate::core::prelude::*;
use crate::input::keyboard::KeyAction;
use crate::output::scroll::ScrollState;

pub struct Message {
    pub content: String,
    pub current_length: usize,
    pub timestamp: Instant,
}

pub struct MessageManager<'a> {
    pub messages: Vec<Message>,
    config: &'a Config,
    pub scroll_state: ScrollState,
}

impl<'a> MessageManager<'a> {
    pub fn new(config: &'a Config) -> Self {
        let scroll_state = ScrollState::new();
        Self {
            messages: Vec::with_capacity(config.max_messages),
            config,
            scroll_state,
        }
    }

    // Neue Methode um die gesamte Content-Höhe zu erhalten
    pub fn get_content_height(&self) -> usize {
        self.messages.len()
    }

    pub fn get_messages(&self) -> Vec<(&String, usize)> {
        let (start, end) = self.scroll_state.get_visible_range();
        let start = start.min(self.messages.len());
        let end = end.min(self.messages.len());

        if start >= end {
            return Vec::new();
        }

        self.messages[start..end]
            .iter()
            .map(|msg| (&msg.content, msg.current_length))
            .collect()
    }

    pub fn add_message(&mut self, content: String) {
        if self.messages.len() >= self.config.max_messages {
            self.messages.remove(0);
            if self.scroll_state.offset > 0 {
                self.scroll_state.offset = self.scroll_state.offset.saturating_sub(1);
            }
        }

        self.messages.push(Message {
            content,
            current_length: 1,
            timestamp: Instant::now(),
        });

        // Erzwinge Auto-Scroll bei neuer Nachricht
        self.scroll_state.force_auto_scroll();

        // Update dimensions with current window height
        self.scroll_state
            .update_dimensions(self.scroll_state.window_height, self.messages.len());
    }

    pub fn handle_scroll(&mut self, action: KeyAction, window_height: usize) {
        // Update dimensions vor dem Scrollen
        self.scroll_state
            .update_dimensions(window_height, self.messages.len());

        match action {
            KeyAction::ScrollUp => {
                self.scroll_state.scroll_up(1);
            }
            KeyAction::ScrollDown => {
                self.scroll_state.scroll_down(1);
            }
            KeyAction::PageUp => {
                let scroll_amount = window_height.saturating_sub(1);
                self.scroll_state.scroll_up(scroll_amount);
            }
            KeyAction::PageDown => {
                let scroll_amount = window_height.saturating_sub(1);
                self.scroll_state.scroll_down(scroll_amount);
            }
            _ => {}
        }
    }

    pub fn get_visible_messages(&self) -> Vec<(&String, usize)> {
        self.get_messages()
    }

    pub fn update_typewriter(&mut self) {
        if let Some(last_message) = self.messages.last_mut() {
            let total_length = last_message.content.graphemes(true).count();

            if last_message.current_length < total_length
                && last_message.timestamp.elapsed() >= self.config.typewriter_delay
            {
                last_message.current_length += 1;
                last_message.timestamp = Instant::now();
            }
        }
    }

    pub fn log(&mut self, level: &str, message: &str) {
        let log_message = format!("[{}] {}", level, message);
        self.add_message(log_message);
    }
}
