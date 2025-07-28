// =====================================================
// FILE: input/input.rs - FINAL VERSION mit RESTART SUPPORT
// =====================================================

use crate::commands::handler::CommandHandler;
use crate::commands::history::{
    HistoryAction, HistoryConfig, HistoryEvent, HistoryEventHandler, HistoryKeyboardHandler,
    HistoryManager,
};
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

pub struct InputState {
    content: String,
    cursor: CursorState,
    prompt: String,
    history_manager: HistoryManager,
    config: Config, // nun eigener Clone
    command_handler: CommandHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_exit_confirmation: bool,
    waiting_for_restart_confirmation: bool, // ← NEU HINZUGEFÜGT
}

/// ✅ Backup structure für InputState
#[derive(Debug, Clone, Default)]
pub struct InputStateBackup {
    pub content: String,
    pub history: Vec<String>,
    pub cursor_pos: usize,
    pub prompt: String,
}

impl InputState {
    pub fn new(prompt: &str, config: &Config) -> Self {
        let history_config = HistoryConfig::from_main_config(config);

        Self {
            content: String::with_capacity(100),
            cursor: CursorState::new(),
            prompt: prompt.to_string(),
            history_manager: HistoryManager::new(history_config.max_entries),
            config: config.clone(), // Clone statt Referenz
            command_handler: CommandHandler::new(),
            keyboard_manager: KeyboardManager::new(),
            waiting_for_exit_confirmation: false,
            waiting_for_restart_confirmation: false, // ← NEU HINZUGEFÜGT
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

    pub fn reset_for_language_change(&mut self) {
        self.waiting_for_exit_confirmation = false;
        self.waiting_for_restart_confirmation = false; // ← NEU HINZUGEFÜGT
        self.content.clear();
        self.history_manager.reset_position();
        self.cursor.move_to_start();
        log::debug!("InputState reset for language change");
    }

    fn handle_exit_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_exit_confirmation = false;

                let confirm_short = crate::i18n::get_translation("system.input.confirm.short", &[]);
                let cancel_short = crate::i18n::get_translation("system.input.cancel.short", &[]);

                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__EXIT__".to_string())
                    }
                    input if input == cancel_short.to_lowercase() => {
                        self.clear_input();
                        Some(crate::i18n::get_translation("system.input.cancelled", &[]))
                    }
                    _ => {
                        self.clear_input();
                        Some(crate::i18n::get_translation("system.input.cancelled", &[]))
                    }
                }
            }
            KeyAction::InsertChar(c) => {
                let confirm_short = crate::i18n::get_translation("system.input.confirm.short", &[]);
                let cancel_short = crate::i18n::get_translation("system.input.cancel.short", &[]);

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
                self.clear_input();
                None
            }
            _ => None,
        }
    }

    // ✅ NEU: Restart Confirmation Handler
    fn handle_restart_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_restart_confirmation = false;

                let confirm_short = crate::i18n::get_translation("system.input.confirm.short", &[]);
                let cancel_short = crate::i18n::get_translation("system.input.cancel.short", &[]);

                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__RESTART__".to_string()) // ✅ RESTART auslösen
                    }
                    input if input == cancel_short.to_lowercase() => {
                        self.clear_input();
                        Some(crate::i18n::get_translation("system.input.cancelled", &[]))
                    }
                    _ => {
                        self.clear_input();
                        Some(crate::i18n::get_translation("system.input.cancelled", &[]))
                    }
                }
            }
            KeyAction::InsertChar(c) => {
                let confirm_short = crate::i18n::get_translation("system.input.confirm.short", &[]);
                let cancel_short = crate::i18n::get_translation("system.input.cancel.short", &[]);

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
                self.clear_input();
                None
            }
            _ => None,
        }
    }

    fn clear_input(&mut self) {
        self.content.clear();
        self.history_manager.reset_position();
        self.cursor.move_to_start();
    }

    fn handle_history_action(&mut self, action: HistoryAction) -> Option<String> {
        match action {
            HistoryAction::NavigatePrevious => {
                if let Some(entry) = self.history_manager.navigate_previous() {
                    self.content = entry;
                    self.cursor.update_text_length(&self.content);
                    self.cursor.move_to_end();
                }
            }
            HistoryAction::NavigateNext => {
                if let Some(entry) = self.history_manager.navigate_next() {
                    self.content = entry;
                    self.cursor.update_text_length(&self.content);
                    self.cursor.move_to_end();
                }
            }
        }
        None
    }

    fn handle_history_event(&mut self, event: HistoryEvent) -> String {
        match event {
            HistoryEvent::Clear => {
                self.history_manager.clear();
                HistoryEventHandler::create_clear_response()
            }
            HistoryEvent::Add(entry) => {
                self.history_manager.add_entry(entry);
                String::new()
            }
            _ => String::new(),
        }
    }

    pub fn execute(&self) -> crate::core::error::Result<String> {
        Ok(format!(
            "__CONFIRM_EXIT__{}",
            get_translation("system.input.confirm_exit", &[])
        ))
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        // ✅ 1. PRÜFE ZUERST auf History-Actions
        if let Some(history_action) = HistoryKeyboardHandler::get_history_action(&key) {
            return self.handle_history_action(history_action);
        }

        // ✅ 2. ESC wird NICHT hier behandelt - nur in ScreenManager!
        if key.code == KeyCode::Esc {
            return None;
        }

        // ✅ 3. NORMALE Keyboard-Actions (ohne ESC!)
        let action = self.keyboard_manager.get_action(&key);

        // ✅ 4. CONFIRMATION HANDLING erweitert
        if self.waiting_for_exit_confirmation {
            return self.handle_exit_confirmation(action);
        }

        if self.waiting_for_restart_confirmation {
            return self.handle_restart_confirmation(action);
        }

        // ✅ 5. NORMALE Eingabeverarbeitung
        match action {
            KeyAction::Submit => {
                if self.content.is_empty() {
                    return None;
                }
                if self.validate_input(&self.content).is_ok() {
                    let content = std::mem::take(&mut self.content);
                    self.cursor.reset_for_empty_text();
                    self.history_manager.add_entry(content.clone());
                    let result = self.command_handler.handle_input(&content);

                    if let Some(event) = HistoryEventHandler::handle_command_result(&result.message)
                    {
                        return Some(self.handle_history_event(event));
                    }

                    if result.message.starts_with("__CONFIRM_EXIT__") {
                        self.waiting_for_exit_confirmation = true;
                        return Some(result.message.replace("__CONFIRM_EXIT__", ""));
                    }

                    // ✅ KORRIGIERT: Restart Confirmation handling (auch mit zusätzlichem Text)
                    if result.message.starts_with("__CONFIRM_RESTART__") {
                        self.waiting_for_restart_confirmation = true;
                        return Some(result.message.replace("__CONFIRM_RESTART__", ""));
                    }

                    // ✅ KRITISCH KORRIGIERT: Restart handling - akzeptiert auch __RESTART__ mit zusätzlichem Text
                    if result.message.starts_with("__RESTART_FORCE__")
                        || result.message.starts_with("__RESTART__")
                    // ✅ DIES WAR DAS PROBLEM!
                    {
                        // ✅ EXTRAHIERE den Text nach __RESTART__ für User-Feedback
                        let feedback_text = if result.message.starts_with("__RESTART_FORCE__") {
                            result
                                .message
                                .replace("__RESTART_FORCE__", "")
                                .trim()
                                .to_string()
                        } else {
                            result.message.replace("__RESTART__", "").trim().to_string()
                        };

                        // ✅ ZEIGE FEEDBACK falls vorhanden
                        if !feedback_text.is_empty() {
                            return Some(format!("__RESTART_WITH_MSG__{}", feedback_text));
                        } else {
                            return Some("__RESTART__".to_string());
                        }
                    }

                    if result.should_exit {
                        return Some(format!("__EXIT__{}", result.message));
                    }
                    return Some(result.message);
                }
                None
            }
            // ✅ REST bleibt unverändert...
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
                if self.content.is_empty() || self.cursor.get_position() == 0 {
                    return None;
                }

                let current_byte_pos = self.cursor.get_byte_position(&self.content);
                let prev_byte_pos = self.cursor.get_prev_byte_position(&self.content);

                if prev_byte_pos >= current_byte_pos || current_byte_pos > self.content.len() {
                    self.cursor.update_text_length(&self.content);
                    return None;
                }

                self.cursor.move_left();
                self.content
                    .replace_range(prev_byte_pos..current_byte_pos, "");
                self.cursor.update_text_length(&self.content);

                if self.content.is_empty() {
                    self.cursor.reset_for_empty_text();
                }
                None
            }
            KeyAction::Delete => {
                let text_length = self.content.graphemes(true).count();
                if self.cursor.get_position() >= text_length || text_length == 0 {
                    return None;
                }

                let current_byte_pos = self.cursor.get_byte_position(&self.content);
                let next_byte_pos = self.cursor.get_next_byte_position(&self.content);

                if current_byte_pos >= next_byte_pos || next_byte_pos > self.content.len() {
                    self.cursor.update_text_length(&self.content);
                    return None;
                }

                self.content
                    .replace_range(current_byte_pos..next_byte_pos, "");
                self.cursor.update_text_length(&self.content);

                if self.content.is_empty() {
                    self.cursor.reset_for_empty_text();
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

    pub fn export_state(&self) -> InputStateBackup {
        InputStateBackup {
            content: self.content.clone(),
            history: self.history_manager.get_all_entries(),
            cursor_pos: self.cursor.get_current_position(),
            prompt: self.prompt.clone(),
        }
    }

    /// ✅ Import state from backup
    pub fn import_state(&mut self, backup: InputStateBackup) {
        // Restore content
        self.content = backup.content;

        // Restore history
        self.history_manager.import_entries(backup.history);

        // Restore cursor
        self.cursor.update_text_length(&self.content);
        // Note: cursor position will be recalculated by update_text_length

        // Restore prompt if changed
        self.prompt = backup.prompt;

        log::debug!(
            "✅ InputState imported: {} chars, {} history entries",
            self.content.len(),
            self.history_manager.entry_count()
        );
    }

    /// ✅ Get current input content (public getter)
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// ✅ Get history count (public getter)
    pub fn get_history_count(&self) -> usize {
        self.history_manager.entry_count()
    }
}

impl Widget for InputState {
    fn render(&self) -> Paragraph {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();
        let mut spans = Vec::with_capacity(4);

        spans.push(Span::styled(
            &self.prompt,
            Style::default().fg(self.config.prompt.color.into()),
        ));

        let prompt_width = self.prompt.graphemes(true).count();
        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10
        } else {
            0
        };

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

        let cursor_char = graphemes.get(cursor_pos).map_or(" ", |&c| c);
        let cursor_style = if self.cursor.is_visible() {
            Style::default()
                .fg(self.config.theme.input_text.into())
                .bg(self.config.theme.cursor.into())
        } else {
            Style::default().fg(self.config.theme.input_text.into())
        };
        spans.push(Span::styled(cursor_char, cursor_style));

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

    /// ✅ NEU: Backup implementation
    fn get_backup_data(&self) -> Option<InputStateBackup> {
        Some(self.export_state())
    }

    /// ✅ NEU: Restore implementation
    fn restore_backup_data(&mut self, backup: InputStateBackup) {
        self.import_state(backup);
    }
}

impl InputWidget for InputState {
    fn update_cursor_blink(&mut self) {
        self.cursor.update_blink();
    }
}
