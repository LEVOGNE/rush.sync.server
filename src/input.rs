// ## FILE: ./src/input.rs
use crate::keyboard::{KeyAction, KeyboardManager};
use crate::prelude::*;

pub struct InputState<'a> {
    content: String,
    cursor_position: usize, // Position in Graphemen, nicht in Bytes
    prompt: String,
    history: Vec<String>,
    history_position: Option<usize>,
    config: &'a Config,
}

impl<'a> InputState<'a> {
    pub fn new(prompt: &str, config: &'a Config) -> Self {
        Self {
            content: String::with_capacity(100), // Vorallokieren für bessere Performance
            cursor_position: 0,
            prompt: prompt.to_string(),
            history: Vec::with_capacity(config.max_history),
            history_position: None,
            config,
        }
    }

    pub fn validate_input(&self, input: &str) -> Result<()> {
        if input.trim().is_empty() {
            return Err(AppError::Validation(
                "Eingabe darf nicht leer sein".to_string(),
            ));
        }

        let grapheme_count = input.graphemes(true).count();
        if grapheme_count > self.config.input_max_length {
            return Err(AppError::Validation(format!(
                "Eingabe zu lang (max {} Zeichen)",
                self.config.input_max_length
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

    fn get_byte_position(&self, grapheme_pos: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .take(grapheme_pos)
            .last()
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(0)
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        let mut keyboard_manager = KeyboardManager::new();
        match keyboard_manager.get_action(&key) {
            KeyAction::Submit => {
                if self.content.is_empty() {
                    return None;
                }
                if let Ok(_) = self.validate_input(&self.content) {
                    let content = std::mem::take(&mut self.content);
                    self.add_to_history(content.clone());
                    self.cursor_position = 0;
                    self.history_position = None;
                    Some(content)
                } else {
                    None
                }
            }

            // Neue Scroll-Aktionen - ignorieren in der Eingabeverarbeitung
            KeyAction::ScrollUp
            | KeyAction::ScrollDown
            | KeyAction::PageUp
            | KeyAction::PageDown => None,

            KeyAction::InsertChar(c) => {
                if self.content.graphemes(true).count() < self.config.input_max_length {
                    let byte_pos = self.get_byte_position(self.cursor_position);
                    self.content.insert(byte_pos, c);
                    self.cursor_position += 1;
                }
                None
            }
            KeyAction::MoveLeft => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                None
            }
            KeyAction::MoveRight => {
                let grapheme_count = self.content.graphemes(true).count();
                if self.cursor_position < grapheme_count {
                    self.cursor_position += 1;
                }
                None
            }
            KeyAction::MoveToStart => {
                self.cursor_position = 0;
                None
            }
            KeyAction::MoveToEnd => {
                self.cursor_position = self.content.graphemes(true).count();
                None
            }
            KeyAction::Backspace => {
                if self.cursor_position > 0 {
                    let prev_byte_pos = self.get_byte_position(self.cursor_position - 1);
                    let current_byte_pos = self.get_byte_position(self.cursor_position);
                    self.content
                        .replace_range(prev_byte_pos..current_byte_pos, "");
                    self.cursor_position -= 1;
                }
                None
            }
            KeyAction::Delete => {
                let grapheme_count = self.content.graphemes(true).count();
                if self.cursor_position < grapheme_count {
                    let current_byte_pos = self.get_byte_position(self.cursor_position);
                    let next_byte_pos = self.get_byte_position(self.cursor_position + 1);
                    self.content
                        .replace_range(current_byte_pos..next_byte_pos, "");
                }
                None
            }
            KeyAction::HistoryPrevious => {
                if let Some(pos) = self.history_position {
                    if pos > 0 {
                        self.history_position = Some(pos - 1);
                        if let Some(entry) = self.history.get(pos - 1) {
                            self.content = entry.clone();
                            self.cursor_position = self.content.graphemes(true).count();
                        }
                    }
                } else if !self.history.is_empty() {
                    self.history_position = Some(self.history.len() - 1);
                    if let Some(entry) = self.history.last() {
                        self.content = entry.clone();
                        self.cursor_position = self.content.graphemes(true).count();
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
                            self.cursor_position = self.content.graphemes(true).count();
                        }
                    } else {
                        self.history_position = None;
                        self.content.clear();
                        self.cursor_position = 0;
                    }
                }
                None
            }
            KeyAction::Cancel | KeyAction::Quit => None,
            KeyAction::ClearLine => {
                self.content.clear();
                self.cursor_position = 0;
                None
            }
            KeyAction::CopySelection | KeyAction::PasteBuffer => None, // Noch nicht implementiert
            KeyAction::NoAction => None,
        }
    }
}

impl<'a> Widget for InputState<'a> {
    fn render(&self) -> Paragraph {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let mut spans = Vec::with_capacity(4);

        // Prompt
        spans.push(Span::styled(
            &self.prompt,
            Style::default().fg(self.config.prompt.color.into()),
        ));

        // Text vor dem Cursor
        if self.cursor_position > 0 {
            let before_cursor = graphemes[..self.cursor_position].join("");
            spans.push(Span::styled(
                before_cursor,
                Style::default().fg(self.config.theme.input_text.into()),
            ));
        }

        // Cursor und Text danach
        if let Some(&cursor_char) = graphemes.get(self.cursor_position) {
            // Cursor-Position
            spans.push(Span::styled(
                cursor_char,
                Style::default()
                    .fg(self.config.theme.input_text.into())
                    .bg(self.config.theme.cursor.into()),
            ));

            // Text nach dem Cursor
            if self.cursor_position < graphemes.len() - 1 {
                let after_cursor = graphemes[self.cursor_position + 1..].join("");
                spans.push(Span::styled(
                    after_cursor,
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        } else {
            // Leerer Cursor am Ende
            spans.push(Span::styled(
                " ",
                Style::default()
                    .fg(self.config.theme.input_text.into())
                    .bg(self.config.theme.cursor.into()),
            ));
        }

        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_style(Style::default().fg(self.config.theme.border.into())),
        )
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<String> {
        self.handle_key_event(key)
    }
}
