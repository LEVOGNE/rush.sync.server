// ## FILE: ./src/input.rs
// src/input/input.rs
use crate::commands::handler::CommandHandler;
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::ui::cursor::CursorState;
use crate::ui::widget::{InputWidget, Widget};

pub struct InputState<'a> {
    content: String,
    cursor: CursorState,
    prompt: String,
    history: Vec<String>,
    history_position: Option<usize>,
    config: &'a Config,
    command_handler: CommandHandler,
    keyboard_manager: KeyboardManager, // NEU
}

impl<'a> InputState<'a> {
    pub fn new(prompt: &str, config: &'a Config) -> Self {
        Self {
            content: String::with_capacity(100),
            cursor: CursorState::new(),
            prompt: prompt.to_string(),
            history: Vec::with_capacity(config.max_history),
            history_position: None,
            config,
            command_handler: CommandHandler::new(),   // NEU
            keyboard_manager: KeyboardManager::new(), // NEU
        }
    }

    pub fn validate_input(&self, input: &str) -> Result<()> {
        if input.trim().is_empty() {
            return Err(AppError::Validation(
                "Eingabe darf nicht leer sein".to_string(),
            ));
        }

        let grapheme_count = input.graphemes(true).count();
        // Erhöhe das Limit auf einen sinnvolleren Wert
        let max_length = 1024; // Oder ein anderer sinnvoller Wert

        if grapheme_count > max_length {
            return Err(AppError::Validation(format!(
                "Eingabe zu lang (max {} Zeichen)",
                max_length
            )));
        }

        Ok(())
    }

    pub fn add_to_history(&mut self, entry: String) {
        // Ignoriere leere Einträge oder Duplikate
        if entry.trim().is_empty() || self.history.contains(&entry) {
            return;
        }

        // Wenn wir das Limit erreichen, entferne den ältesten Eintrag
        if self.history.len() >= self.config.max_history {
            self.history.remove(0);
        }

        self.history.push(entry);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        match self.keyboard_manager.get_action(&key) {
            KeyAction::Submit => {
                if self.content.is_empty() {
                    return None;
                }
                if self.validate_input(&self.content).is_ok() {
                    let content = std::mem::take(&mut self.content);
                    self.add_to_history(content.clone());
                    self.cursor.move_to_start();
                    self.history_position = None;

                    // Verarbeite den Befehl über den CommandHandler
                    let result = self.command_handler.handle_input(&content);

                    // Falls Exit gewünscht ist, markieren wir die Nachricht mit einem speziellen Präfix
                    if result.should_exit {
                        return Some(format!("__EXIT__{}", result.message));
                    }
                    return Some(result.message);
                } else {
                    return None;
                }
            }
            KeyAction::InsertChar(c) => {
                if self.content.graphemes(true).count() < self.config.input_max_length {
                    let byte_pos = self.cursor.get_byte_position(&self.content);
                    self.content.insert(byte_pos, c);
                    self.cursor.update_text_length(&self.content);
                    self.cursor.move_right();
                }
                None
            }
            KeyAction::MoveLeft => {
                self.cursor.move_left();
                None
            }
            KeyAction::MoveRight => {
                self.cursor.move_right();
                None
            }
            KeyAction::MoveToStart => {
                self.cursor.move_to_start();
                None
            }
            KeyAction::MoveToEnd => {
                self.cursor.move_to_end();
                None
            }
            KeyAction::Backspace => {
                // Schneller Early Exit wenn nichts zu löschen ist
                if self.content.is_empty() || self.cursor.get_position() == 0 {
                    return None;
                }

                log::debug!(
                    "START Backspace - Text: '{}', Position: {}",
                    self.content,
                    self.cursor.get_position()
                );

                // Rest der Backspace-Logik...
                let current_byte_pos = self.cursor.get_byte_position(&self.content);
                let prev_byte_pos = self.cursor.get_prev_byte_position(&self.content);

                log::debug!(
                    "Backspace - Current byte pos: {}, Prev byte pos: {}, Text: '{}'",
                    current_byte_pos,
                    prev_byte_pos,
                    self.content
                );

                self.cursor.move_left();
                self.content
                    .replace_range(prev_byte_pos..current_byte_pos, "");

                log::debug!(
                    "After Backspace - Text: '{}', New Position: {}",
                    self.content,
                    self.cursor.get_position()
                );

                self.cursor.update_text_length(&self.content);
                None
            }
            KeyAction::Delete => {
                if self.cursor.get_position() < self.content.graphemes(true).count() {
                    let current_byte_pos = self.cursor.get_byte_position(&self.content);
                    let next_byte_pos = self.cursor.get_next_byte_position(&self.content);
                    self.content
                        .replace_range(current_byte_pos..next_byte_pos, "");
                    self.cursor.update_text_length(&self.content);
                }
                None
            }
            KeyAction::HistoryPrevious => {
                if let Some(pos) = self.history_position {
                    if pos > 0 {
                        self.history_position = Some(pos - 1);
                        if let Some(entry) = self.history.get(pos - 1) {
                            self.content = entry.clone();
                            self.cursor.update_text_length(&self.content);
                            self.cursor.move_to_end();
                        }
                    }
                } else if !self.history.is_empty() {
                    self.history_position = Some(self.history.len() - 1);
                    if let Some(entry) = self.history.last() {
                        self.content = entry.clone();
                        self.cursor.update_text_length(&self.content);
                        self.cursor.move_to_end();
                    }
                }
                None
            }
            KeyAction::HistoryNext => {
                if let Some(pos) = self.history_position {
                    if pos < self.history.len() - 1 {
                        self.history_position = Some(pos + 1);
                        if let Some(entry) = self.history.get(pos + 1) {
                            self.content = entry.clone();
                            self.cursor.update_text_length(&self.content);
                            self.cursor.move_to_end();
                        }
                    } else {
                        self.history_position = None;
                        self.content.clear();
                        self.cursor.move_to_start();
                    }
                }
                None
            }
            KeyAction::ClearLine => {
                self.content.clear();
                self.cursor.move_to_start();
                None
            }
            // Neue match arms für die fehlenden Aktionen
            KeyAction::ScrollUp
            | KeyAction::ScrollDown
            | KeyAction::PageUp
            | KeyAction::PageDown
            | KeyAction::Cancel
            | KeyAction::Quit
            | KeyAction::CopySelection
            | KeyAction::PasteBuffer
            | KeyAction::NoAction => None,
        }
    }
}

impl<'a> Widget for InputState<'a> {
    fn render(&self) -> Paragraph {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();
        let mut spans = Vec::with_capacity(4);

        // Prompt hinzufügen
        spans.push(Span::styled(
            &self.prompt,
            Style::default().fg(self.config.prompt.color.into()),
        ));

        // Berechne die verfügbare Breite für den Text
        // Wir subtrahieren die Länge des Prompts und einen Puffer von 4 Zeichen
        let prompt_width = self.prompt.graphemes(true).count();
        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        // Berechne den sichtbaren Bereich basierend auf der Cursor-Position
        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10 // 10 Zeichen Puffer für bessere Lesbarkeit
        } else {
            0
        };

        // Rendere den Text vor dem Cursor
        if cursor_pos > 0 {
            let visible_text = if viewport_start < cursor_pos {
                graphemes[viewport_start..cursor_pos].join("")
            } else {
                String::new()
            };

            spans.push(Span::styled(
                visible_text,
                Style::default().fg(self.config.theme.input_text.into()),
            ));
        }

        // Cursor-Zeichen rendern
        let cursor_char = graphemes.get(cursor_pos).map_or(" ", |&c| c);
        let cursor_style = if self.cursor.is_visible() {
            Style::default()
                .fg(self.config.theme.input_text.into())
                .bg(self.config.theme.cursor.into())
        } else {
            Style::default().fg(self.config.theme.input_text.into())
        };
        spans.push(Span::styled(cursor_char, cursor_style));

        // Text nach dem Cursor
        if cursor_pos < graphemes.len() {
            let remaining_width = available_width.saturating_sub(cursor_pos - viewport_start);
            let end_pos = (cursor_pos + 1 + remaining_width).min(graphemes.len());

            if cursor_pos + 1 < end_pos {
                spans.push(Span::styled(
                    graphemes[cursor_pos + 1..end_pos].join(""),
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        }

        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .padding(Padding::new(3, 1, 1, 1))
                .borders(Borders::NONE)
                .style(Style::default().bg(self.config.theme.input_bg.into())),
        )
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<String> {
        self.handle_key_event(key)
    }

    fn as_input_state(&mut self) -> Option<&mut dyn InputWidget> {
        Some(self)
    }
}

impl<'a> InputWidget for InputState<'a> {
    fn update_cursor_blink(&mut self) {
        self.cursor.update_blink();
    }
}
