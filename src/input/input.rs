use crate::commands::handler::CommandHandler;
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::ui::cursor::CursorState;
use crate::ui::widget::{InputWidget, Widget};
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;

pub struct InputState<'a> {
    content: String,
    cursor: CursorState,
    prompt: String,
    history: Vec<String>,
    history_position: Option<usize>,
    config: &'a Config,
    command_handler: CommandHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_exit_confirmation: bool,
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
            command_handler: CommandHandler::new(),
            keyboard_manager: KeyboardManager::new(),
            waiting_for_exit_confirmation: false,
        }
    }

    pub fn validate_input(&self, input: &str) -> crate::core::error::Result<()> {
        if input.trim().is_empty() {
            return Err(AppError::Validation(get_translation(
                "system.input.empty",
                &[],
            )));
        }

        let grapheme_count = input.graphemes(true).count();
        let max_length = 1024;

        if grapheme_count > max_length {
            return Err(AppError::Validation(get_translation(
                "system.input.too_long",
                &[&max_length.to_string()],
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

    /*    fn handle_history_navigation(&mut self, action: KeyAction) -> Option<String> {
           match action {
               KeyAction::HistoryPrevious => {
                   if let Some(pos) = self.history_position {
                       if pos > 0 {
                           self.update_history_position(pos - 1);
                       }
                   } else if !self.history.is_empty() {
                       self.update_history_position(self.history.len() - 1);
                   }
               }
               KeyAction::HistoryNext => {
                   if let Some(pos) = self.history_position {
                       if pos < self.history.len() - 1 {
                           self.update_history_position(pos + 1);
                       } else {
                           self.clear_history_position();
                       }
                   }
               }
               _ => {}
           }
           None
       }

       fn update_history_position(&mut self, pos: usize) {
           if let Some(entry) = self.history.get(pos) {
               self.content = entry.clone();
               self.cursor.update_text_length(&self.content);
               self.cursor.move_to_end();
               self.history_position = Some(pos);
           }
       }
    */
    fn clear_history_position(&mut self) {
        self.history_position = None;
        self.content.clear();
        self.cursor.move_to_start();
    }

    fn handle_exit_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_exit_confirmation = false;
                let confirm_short = get_translation("system.input.confirm.short", &[]);
                let cancel_short = get_translation("system.input.cancel.short", &[]);

                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__EXIT__".to_string())
                    }
                    input if input == cancel_short.to_lowercase() => {
                        self.clear_history_position();
                        Some(get_translation("system.input.cancelled", &[]))
                    }
                    _ => {
                        self.clear_history_position();
                        Some(get_translation("system.input.cancelled", &[]))
                    }
                }
            }
            KeyAction::InsertChar(c) => {
                let confirm_short = get_translation("system.input.confirm.short", &[]);
                let cancel_short = get_translation("system.input.cancel.short", &[]);

                if c.to_lowercase().to_string() == confirm_short.to_lowercase()
                    || c.to_lowercase().to_string() == cancel_short.to_lowercase()
                {
                    self.content.clear();
                    self.content.push(c);
                    self.cursor.update_text_length(&self.content);
                    self.cursor.move_to_end();
                }
                None
            }
            KeyAction::Backspace | KeyAction::Delete | KeyAction::ClearLine => {
                self.clear_history_position();
                None
            }
            _ => None,
        }
    }

    pub fn execute(&self) -> crate::core::error::Result<String> {
        Ok(format!(
            "__CONFIRM_EXIT__{}",
            get_translation("system.input.confirm_exit", &[])
        ))
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        let action = self.keyboard_manager.get_action(&key);

        if self.waiting_for_exit_confirmation {
            return self.handle_exit_confirmation(action);
        }

        // Normale Eingabeverarbeitung
        match action {
            KeyAction::Submit => {
                if self.content.is_empty() {
                    return None;
                }
                if self.validate_input(&self.content).is_ok() {
                    let content = std::mem::take(&mut self.content);
                    self.add_to_history(content.clone());
                    self.cursor.move_to_start();
                    self.history_position = None;

                    let result = self.command_handler.handle_input(&content);

                    // Prüfe auf History-Clear Befehl
                    if result.message == "__CLEAR_HISTORY__" {
                        self.history.clear();
                        self.history_position = None;
                        return Some("History wurde gelöscht".to_string());
                    }

                    if result.message.starts_with("__CONFIRM_EXIT__") {
                        self.waiting_for_exit_confirmation = true;
                        return Some(result.message.replace("__CONFIRM_EXIT__", ""));
                    }

                    if result.should_exit {
                        return Some(format!("__EXIT__{}", result.message));
                    }
                    return Some(result.message);
                }
                None
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
            KeyAction::Backspace => {
                if self.content.is_empty() || self.cursor.get_position() == 0 {
                    return None;
                }
                let current_byte_pos = self.cursor.get_byte_position(&self.content);
                let prev_byte_pos = self.cursor.get_prev_byte_position(&self.content);

                self.cursor.move_left();
                self.content
                    .replace_range(prev_byte_pos..current_byte_pos, "");
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

            KeyAction::ClearLine
            | KeyAction::ScrollUp
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

impl Widget for InputState<'_> {
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

impl InputWidget for InputState<'_> {
    fn update_cursor_blink(&mut self) {
        self.cursor.update_blink();
    }
}
