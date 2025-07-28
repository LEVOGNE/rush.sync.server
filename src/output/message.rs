// =====================================================
// FILE: src/output/message.rs - OWNED CONFIG SUPPORT
// =====================================================

use crate::core::prelude::*;
use crate::input::keyboard::KeyAction;
use crate::output::scroll::ScrollState;
use unicode_segmentation::UnicodeSegmentation;

pub struct Message {
    pub content: String,
    pub current_length: usize,
    pub timestamp: Instant,
}

pub struct MessageManager {
    pub messages: Vec<Message>,
    config: Config, // ✅ OWNED statt &'a Config
    pub scroll_state: ScrollState,
}

impl MessageManager {
    /// ✅ NEUER CONSTRUCTOR mit owned config
    pub fn new(config: &Config) -> Self {
        let scroll_state = ScrollState::new();
        Self {
            messages: Vec::with_capacity(config.max_messages),
            config: config.clone(), // ✅ CLONE die Config
            scroll_state,
        }
    }

    /// ✅ NEU: UPDATE METHOD für Live-Changes
    pub fn update_config(&mut self, new_config: &Config) {
        self.config = new_config.clone();

        // ✅ RESIZE messages buffer falls max_messages geändert wurde
        if self.messages.len() > self.config.max_messages {
            let excess = self.messages.len() - self.config.max_messages;
            self.messages.drain(0..excess);

            // ✅ UPDATE scroll state nach message removal
            if self.scroll_state.offset > 0 {
                self.scroll_state.offset = self.scroll_state.offset.saturating_sub(excess);
            }
        } else {
            // ✅ RESERVE mehr Platz falls max_messages erhöht wurde
            self.messages
                .reserve(self.config.max_messages.saturating_sub(self.messages.len()));
        }

        log::debug!(
            "MessageManager config updated: max_messages = {}, typewriter_delay = {}ms",
            self.config.max_messages,
            self.config.typewriter_delay.as_millis()
        );
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_state.force_auto_scroll();
    }

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
        // ✅ BUFFER-MANAGEMENT: Entferne alte Messages wenn Buffer voll
        if self.messages.len() >= self.config.max_messages {
            self.messages.remove(0);
            if self.scroll_state.offset > 0 {
                self.scroll_state.offset = self.scroll_state.offset.saturating_sub(1);
            }
        }

        // ✅ TYPEWRITER FIX: Wenn delay = 0, sofort alles anzeigen
        let initial_length = if self.config.typewriter_delay.as_millis() == 0 {
            content.graphemes(true).count() // ✅ Komplette Nachricht sofort
        } else {
            1 // ✅ Typewriter-Effekt: nur erstes Zeichen
        };

        self.messages.push(Message {
            content,
            current_length: initial_length,
            timestamp: Instant::now(),
        });

        // ✅ ERZWINGE Auto-Scroll bei neuer Nachricht
        self.scroll_state.force_auto_scroll();

        // ✅ UPDATE dimensions mit aktueller window height
        self.scroll_state
            .update_dimensions(self.scroll_state.window_height, self.messages.len());
    }

    pub fn handle_scroll(&mut self, action: KeyAction, window_height: usize) {
        // ✅ UPDATE dimensions vor dem Scrollen
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

    /// ✅ HAUPTFIX: Typewriter Update nur wenn delay > 0
    pub fn update_typewriter(&mut self) {
        // ✅ EARLY RETURN: Wenn typewriter_delay = 0, mache nichts
        if self.config.typewriter_delay.as_millis() == 0 {
            return;
        }

        if let Some(last_message) = self.messages.last_mut() {
            let total_length = last_message.content.graphemes(true).count();

            if last_message.current_length < total_length {
                let elapsed = last_message.timestamp.elapsed();

                // ✅ ULTRASCHNELL: Mehrere Zeichen pro Update bei sehr niedrigen Delays
                if elapsed >= self.config.typewriter_delay {
                    let chars_to_add = if self.config.typewriter_delay.as_millis() <= 5 {
                        let ratio = elapsed.as_millis() as f64
                            / self.config.typewriter_delay.as_millis() as f64;
                        ratio.floor().max(1.0) as usize
                    } else {
                        1
                    };

                    let new_length = (last_message.current_length + chars_to_add).min(total_length);
                    last_message.current_length = new_length;
                    last_message.timestamp = Instant::now();
                }
            }
        }
    }

    pub fn log(&mut self, level: &str, message: &str) {
        let log_message = format!("[{}] {}", level, message);
        self.add_message(log_message);
    }
}
