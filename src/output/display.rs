// =====================================================
// FILE: src/output/display.rs - MESSAGE + OUTPUT KOMBINIERT
// =====================================================

use crate::core::prelude::*;
use crate::input::keyboard::KeyAction;
use crate::output::scroll::ScrollState;
use crate::ui::color::AppColor;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

// ✅ MESSAGE STRUKTUR
#[derive(Debug)]
pub struct Message {
    pub content: String,
    pub current_length: usize,
    pub timestamp: Instant,
}

// ✅ KOMBINIERTES DISPLAY-SYSTEM
pub struct MessageDisplay {
    messages: Vec<Message>,
    config: Config,
    pub scroll_state: ScrollState,
}

impl MessageDisplay {
    /// Erstellt neues MessageDisplay
    pub fn new(config: &Config) -> Self {
        Self {
            messages: Vec::with_capacity(config.max_messages),
            config: config.clone(),
            scroll_state: ScrollState::new(),
        }
    }

    /// Updates Config (für Live-Updates)
    pub fn update_config(&mut self, new_config: &Config) {
        self.config = new_config.clone();

        // Resize messages buffer falls max_messages geändert wurde
        if self.messages.len() > self.config.max_messages {
            let excess = self.messages.len() - self.config.max_messages;
            self.messages.drain(0..excess);

            if self.scroll_state.offset > 0 {
                self.scroll_state.offset = self.scroll_state.offset.saturating_sub(excess);
            }
        } else {
            self.messages
                .reserve(self.config.max_messages.saturating_sub(self.messages.len()));
        }

        log::debug!(
            "MessageDisplay config updated: max_messages = {}, typewriter_delay = {}ms",
            self.config.max_messages,
            self.config.typewriter_delay.as_millis()
        );
    }

    /// Löscht alle Messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_state.force_auto_scroll();
    }

    /// Fügt neue Message hinzu
    pub fn add_message(&mut self, content: String) {
        // Buffer-Management
        if self.messages.len() >= self.config.max_messages {
            self.messages.remove(0);
        }

        // Typewriter
        let initial_length = if self.config.typewriter_delay.as_millis() == 0 {
            content.graphemes(true).count()
        } else {
            1
        };

        self.messages.push(Message {
            content,
            current_length: initial_length,
            timestamp: Instant::now(),
        });

        // ✅ KORRIGIERT: Scroll-Update mit richtiger Berechnung
        let total_lines = self.calculate_total_lines();
        self.scroll_state
            .update_dimensions(self.scroll_state.window_height, total_lines);
        self.scroll_state.force_auto_scroll();

        // log::trace!(
        //     "📨 Message added, total_lines={}, window_height={}",
        //     total_lines,
        //     self.scroll_state.window_height
        // );
    }

    /// Typewriter Update
    pub fn update_typewriter(&mut self) {
        if self.config.typewriter_delay.as_millis() == 0 {
            return;
        }

        if let Some(last_message) = self.messages.last_mut() {
            let total_length = last_message.content.graphemes(true).count();

            if last_message.current_length < total_length {
                let elapsed = last_message.timestamp.elapsed();

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

    /// Handle Scroll Events
    pub fn handle_scroll(&mut self, action: KeyAction, window_height: usize) {
        let total_lines = self.calculate_total_lines();
        self.scroll_state
            .update_dimensions(window_height, total_lines);

        match action {
            KeyAction::ScrollUp => self.scroll_state.scroll_up(1),
            KeyAction::ScrollDown => self.scroll_state.scroll_down(1),
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

    fn calculate_total_lines(&self) -> usize {
        let total = self
            .messages
            .iter()
            .map(|msg| {
                let clean_msg = clean_message_for_display(&msg.content);
                let line_count = if clean_msg.is_empty() {
                    1
                } else {
                    clean_msg.lines().count()
                };
                line_count.max(1) // Mindestens 1 Zeile
            })
            .sum::<usize>();

        //log::trace!("📊 Total lines calculated: {}", total);
        total
    }

    /// Content Height für Scrolling
    pub fn get_content_height(&self) -> usize {
        self.calculate_total_lines()
    }

    /// Sichtbare Messages für Rendering
    pub fn get_visible_messages(&self) -> Vec<(&String, usize)> {
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

    /// Log Helper
    pub fn log(&mut self, level: &str, message: &str) {
        let log_message = format!("[{}] {}", level, message);
        self.add_message(log_message);
    }

    /// Erstellt Output-Widget für Rendering
    pub fn create_output_widget_for_rendering(
        &self,
        _available_height: u16,
    ) -> (Vec<(String, usize)>, Config) {
        let messages = self.get_visible_messages();
        let messages_owned: Vec<(String, usize)> = messages
            .into_iter()
            .map(|(content, length)| (content.clone(), length))
            .collect();
        (messages_owned, self.config.clone())
    }

    /// ✅ NEU: Getter für messages (für Debug)
    pub fn get_messages_count(&self) -> usize {
        self.messages.len()
    }
}

// ✅ OUTPUT WIDGET CREATION (aus output.rs übernommen)

/// Entfernt ANSI-Codes aus Logs
fn clean_ansi_codes(message: &str) -> String {
    String::from_utf8_lossy(&strip(message.as_bytes()).unwrap_or_default()).into_owned()
}

/// Entfernt interne Steuerzeichen
fn clean_message_for_display(message: &str) -> String {
    clean_ansi_codes(message)
        .replace("__CONFIRM_EXIT__", "")
        .replace("__CLEAR__", "")
        .trim()
        .to_string()
}

/// Teilt Message in Text + Marker
fn parse_message_parts(message: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut chars = message.char_indices().peekable();
    let mut start = 0;

    while let Some((i, c)) = chars.peek().cloned() {
        if c == '[' {
            if start < i {
                let text = &message[start..i];
                if !text.trim().is_empty() {
                    parts.push((text.to_owned(), false));
                }
            }

            if let Some(end_idx) = message[i..].find(']') {
                let end = i + end_idx + 1;
                parts.push((message[i..end].to_owned(), true));
                start = end;
                while let Some(&(ci, _)) = chars.peek() {
                    if ci < end {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                parts.push((message[i..].to_owned(), false));
                break;
            }
        } else {
            chars.next();
        }
    }

    if start < message.len() {
        let remaining = &message[start..];
        if !remaining.trim().is_empty() {
            parts.push((remaining.to_owned(), false));
        }
    }

    if parts.is_empty() {
        parts.push((message.to_owned(), false));
    }

    parts
}

/// Marker-Farben
fn get_marker_color(marker: &str) -> AppColor {
    let display_category = marker
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_start_matches("cat:")
        .to_lowercase();

    if AppColor::from_any(&display_category).to_name() != "gray" {
        return AppColor::from_any(&display_category);
    }

    let mapped_category = crate::i18n::get_color_category_for_display(&display_category);
    AppColor::from_any(mapped_category)
}

/// Hauptfunktion: Baut den fertigen Paragraph
pub fn create_output_widget<'a>(
    messages: &'a [(&'a String, usize)],
    available_height: u16,
    config: &Config,
) -> Paragraph<'a> {
    let mut lines = Vec::new();
    let max_lines = available_height as usize; // ✅ KEINE -1 mehr!

    // ✅ CRITICAL CHECK
    if max_lines == 0 {
        return Paragraph::new(vec![Line::from(vec![Span::raw("⚠️ NO SPACE")])]).block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        );
    }

    if messages.is_empty() {
        let empty_lines = vec![Line::from(vec![Span::raw("")]); max_lines];
        return Paragraph::new(empty_lines)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(config.theme.output_bg.into())),
            )
            .wrap(Wrap { trim: true });
    }

    // ✅ MULTILINE PROCESSING - sauber ohne Debug-Spam
    for (message_idx, (message, current_length)) in messages.iter().enumerate() {
        let is_last_message = message_idx == messages.len() - 1;
        let clean_message = clean_message_for_display(message);
        let message_lines: Vec<&str> = clean_message.lines().collect();

        if message_lines.is_empty() {
            lines.push(Line::from(vec![Span::raw("")]));
        } else {
            for (line_idx, line_content) in message_lines.iter().enumerate() {
                if lines.len() >= max_lines {
                    break; // ✅ Harte Grenze
                }

                let is_last_line = line_idx == message_lines.len() - 1;

                let visible_chars = if is_last_message && is_last_line {
                    let chars_before_this_line: usize = message_lines
                        .iter()
                        .take(line_idx)
                        .map(|l| l.graphemes(true).count() + 1)
                        .sum();

                    let available_for_this_line =
                        current_length.saturating_sub(chars_before_this_line);
                    available_for_this_line.min(line_content.graphemes(true).count())
                } else {
                    line_content.graphemes(true).count()
                };

                let message_parts = parse_message_parts(line_content);
                let mut spans = Vec::new();
                let mut chars_used = 0;

                for (part_text, is_marker) in message_parts {
                    let part_chars = part_text.graphemes(true).count();
                    let part_style = if is_marker {
                        Style::default().fg(get_marker_color(&part_text).into())
                    } else {
                        Style::default().fg(config.theme.output_text.into())
                    };

                    if chars_used >= visible_chars {
                        break;
                    }

                    let chars_needed = visible_chars - chars_used;

                    if chars_needed >= part_chars {
                        spans.push(Span::styled(part_text, part_style));
                        chars_used += part_chars;
                    } else {
                        let graphemes: Vec<&str> = part_text.graphemes(true).collect();
                        spans.push(Span::styled(
                            graphemes
                                .iter()
                                .take(chars_needed)
                                .copied()
                                .collect::<String>(),
                            part_style,
                        ));
                        break;
                    }
                }

                if spans.is_empty() {
                    spans.push(Span::raw(""));
                }

                lines.push(Line::from(spans));
            }
        }

        if lines.len() >= max_lines {
            break;
        }
    }

    // ✅ FILL remaining lines nur wenn nötig
    while lines.len() < max_lines {
        lines.push(Line::from(vec![Span::raw("")]));
    }

    // ✅ FINAL SAFETY
    lines.truncate(max_lines);

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        )
        .wrap(Wrap { trim: true })
}
