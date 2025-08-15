// =====================================================
// FILE: src/input/state.rs - CLEANED VERSION ohne Debug Commands
// =====================================================

use crate::commands::handler::CommandHandler;
use crate::commands::history::{
    HistoryAction, HistoryConfig, HistoryEvent, HistoryEventHandler, HistoryKeyboardHandler,
    HistoryManager,
};
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::ui::cursor::{CursorKind, UiCursor};
use crate::ui::widget::{InputWidget, Widget};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use unicode_segmentation::UnicodeSegmentation;

pub struct InputState {
    content: String,
    cursor: UiCursor,
    prompt: String,
    history_manager: HistoryManager,
    config: Config,
    command_handler: CommandHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_exit_confirmation: bool,
    waiting_for_restart_confirmation: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InputStateBackup {
    pub content: String,
    pub history: Vec<String>,
    pub cursor_pos: usize,
}

impl InputState {
    pub fn new(config: &Config) -> Self {
        let history_config = HistoryConfig::from_main_config(config);

        Self {
            content: String::with_capacity(100),
            cursor: UiCursor::from_config(config, CursorKind::Input),
            prompt: config.theme.input_cursor_prefix.clone(),
            history_manager: HistoryManager::new(history_config.max_entries),
            config: config.clone(),
            command_handler: CommandHandler::new(),
            keyboard_manager: KeyboardManager::new(),
            waiting_for_exit_confirmation: false,
            waiting_for_restart_confirmation: false,
        }
    }

    pub fn update_from_config(&mut self, config: &Config) {
        self.cursor.update_from_config(config);
        self.prompt = config.theme.input_cursor_prefix.clone();
        self.config = config.clone();
    }

    pub fn validate_input(&self, input: &str) -> crate::core::error::Result<()> {
        if input.trim().is_empty() {
            return Err(AppError::Validation(t!("system.input.empty")));
        }
        let grapheme_count = input.graphemes(true).count();
        let max_length = 1024;

        if grapheme_count > max_length {
            return Err(AppError::Validation(t!(
                "system.input.too_long",
                &max_length.to_string()
            )));
        }
        Ok(())
    }

    pub fn reset_for_language_change(&mut self) {
        self.waiting_for_exit_confirmation = false;
        self.waiting_for_restart_confirmation = false;
        self.content.clear();
        self.history_manager.reset_position();
        self.cursor.move_to_start();
    }

    fn handle_exit_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_exit_confirmation = false;
                let confirm_short = t!("system.input.confirm.short");
                let cancel_short = t!("system.input.cancel.short");
                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__EXIT__".to_string())
                    }
                    input if input == cancel_short.to_lowercase() => {
                        self.clear_input();
                        Some(t!("system.input.cancelled"))
                    }
                    _ => {
                        self.clear_input();
                        Some(t!("system.input.cancelled"))
                    }
                }
            }
            KeyAction::InsertChar(c) => {
                let confirm_short = t!("system.input.confirm.short");
                let cancel_short = t!("system.input.cancel.short");
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

    fn handle_restart_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_restart_confirmation = false;
                let confirm_short = t!("system.input.confirm.short");
                let cancel_short = t!("system.input.cancel.short");
                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__RESTART__".to_string())
                    }
                    input if input == cancel_short.to_lowercase() => {
                        self.clear_input();
                        Some(t!("system.input.cancelled"))
                    }
                    _ => {
                        self.clear_input();
                        Some(t!("system.input.cancelled"))
                    }
                }
            }
            KeyAction::InsertChar(c) => {
                let confirm_short = t!("system.input.confirm.short");
                let cancel_short = t!("system.input.cancel.short");
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
            t!("system.input.confirm_exit")
        ))
    }

    fn read_clipboard(&self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("pbpaste")
                .output()
                .ok()
                .and_then(|output| {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(text.trim().to_string())
                    }
                })
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
                .or_else(|_| {
                    std::process::Command::new("xsel")
                        .args(["-b", "-o"])
                        .output()
                })
                .ok()
                .and_then(|output| {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(text.trim().to_string())
                    }
                })
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("powershell")
                .args(["-Command", "Get-Clipboard"])
                .output()
                .ok()
                .and_then(|output| {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(text.trim().to_string())
                    }
                })
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        None
    }

    // ✅ EINFACHES CLIPBOARD SCHREIBEN
    fn write_clipboard(&self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait()
                })
                .is_ok()
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait()
                })
                .is_ok()
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", &format!("echo {}| clip", text)])
                .output()
                .is_ok()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        false
    }

    // ✅ NEUE SELECTION HANDLING
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        if let Some(history_action) = HistoryKeyboardHandler::get_history_action(&key) {
            return self.handle_history_action(history_action);
        }

        if key.code == KeyCode::Esc {
            return None;
        }

        let action = self.keyboard_manager.get_action(&key);

        if self.waiting_for_exit_confirmation {
            return self.handle_exit_confirmation(action);
        }
        if self.waiting_for_restart_confirmation {
            return self.handle_restart_confirmation(action);
        }

        match action {
            // ✅ FIXED: PASTE - Kompletten Text einfügen
            KeyAction::PasteBuffer => {
                if let Some(clipboard_text) = self.read_clipboard() {
                    // Text säubern: Keine Newlines, nur druckbare Zeichen
                    let clean_text = clipboard_text
                        .replace('\n', " ")
                        .replace('\r', "")
                        .replace('\t', " ")
                        .chars()
                        .filter(|c| !c.is_control() || *c == ' ')
                        .collect::<String>();

                    if !clean_text.is_empty() {
                        // Verfügbaren Platz berechnen
                        let current_length = self.content.graphemes(true).count();
                        let available_space =
                            self.config.input_max_length.saturating_sub(current_length);

                        // Text kürzen falls nötig
                        let paste_text = if clean_text.graphemes(true).count() > available_space {
                            clean_text
                                .graphemes(true)
                                .take(available_space)
                                .collect::<String>()
                        } else {
                            clean_text
                        };

                        if !paste_text.is_empty() {
                            // An Cursor-Position einfügen
                            let byte_pos = self.cursor.get_byte_position(&self.content);
                            self.content.insert_str(byte_pos, &paste_text);

                            // Cursor nach eingefügtem Text positionieren
                            let chars_added = paste_text.graphemes(true).count();
                            self.cursor.update_text_length(&self.content);
                            for _ in 0..chars_added {
                                self.cursor.move_right();
                            }

                            return Some(format!("📋 Pasted {} chars", chars_added));
                        }
                    }
                }
                Some("❌ Clipboard empty or invalid".to_string())
            }

            // ✅ FIXED: COPY - Ganzen Input kopieren oder Selection
            KeyAction::CopySelection => {
                if !self.content.is_empty() {
                    if self.write_clipboard(&self.content) {
                        Some(format!(
                            "📋 Copied: \"{}\"",
                            if self.content.len() > 50 {
                                format!("{}...", &self.content[..50])
                            } else {
                                self.content.clone()
                            }
                        ))
                    } else {
                        Some("❌ Copy failed".to_string())
                    }
                } else {
                    Some("❌ Nothing to copy".to_string())
                }
            }

            // Rest bleibt gleich...
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
                    if result.message.starts_with("__CONFIRM_RESTART__") {
                        self.waiting_for_restart_confirmation = true;
                        return Some(result.message.replace("__CONFIRM_RESTART__", ""));
                    }
                    if result.message.starts_with("__RESTART_FORCE__")
                        || result.message.starts_with("__RESTART__")
                    {
                        let feedback_text = if result.message.starts_with("__RESTART_FORCE__") {
                            result
                                .message
                                .replace("__RESTART_FORCE__", "")
                                .trim()
                                .to_string()
                        } else {
                            result.message.replace("__RESTART__", "").trim().to_string()
                        };
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

            KeyAction::InsertChar(c) => {
                if self.content.graphemes(true).count() < self.config.input_max_length {
                    let byte_pos = self.cursor.get_byte_position(&self.content);
                    self.content.insert(byte_pos, c);
                    self.cursor.update_text_length(&self.content);
                    self.cursor.move_right();
                }
                None
            }

            KeyAction::ClearLine => {
                if !self.content.is_empty() {
                    // Vor dem Löschen in Clipboard kopieren
                    if self.write_clipboard(&self.content) {
                        let old_content = self.content.clone();
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        self.history_manager.reset_position();
                        Some(format!(
                            "📋 Cut: \"{}\"",
                            if old_content.len() > 50 {
                                format!("{}...", &old_content[..50])
                            } else {
                                old_content
                            }
                        ))
                    } else {
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        self.history_manager.reset_position();
                        Some("Input cleared".to_string())
                    }
                } else {
                    None
                }
            }

            // Cursor movement...
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

            KeyAction::ScrollUp
            | KeyAction::ScrollDown
            | KeyAction::PageUp
            | KeyAction::PageDown
            | KeyAction::Cancel
            | KeyAction::Quit
            | KeyAction::NoAction => None,
        }
    }

    pub fn export_state(&self) -> InputStateBackup {
        InputStateBackup {
            content: self.content.clone(),
            history: self.history_manager.get_all_entries(),
            cursor_pos: self.cursor.get_current_position(),
        }
    }

    pub fn import_state(&mut self, backup: InputStateBackup) {
        self.content = backup.content;
        self.history_manager.import_entries(backup.history);
        self.cursor.update_text_length(&self.content);
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_history_count(&self) -> usize {
        self.history_manager.entry_count()
    }
}

impl Widget for InputState {
    fn render(&self) -> Paragraph {
        self.render_with_cursor().0
    }

    fn render_with_cursor(&self) -> (Paragraph, Option<(u16, u16)>) {
        use unicode_width::UnicodeWidthStr;

        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();

        let prompt_display = self.config.theme.input_cursor_prefix.clone();
        let prompt_width = prompt_display.width();

        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10
        } else {
            0
        };

        let mut spans = Vec::new();
        spans.push(Span::styled(
            prompt_display,
            Style::default().fg(self.config.theme.input_cursor_color.into()),
        ));

        let end_pos = (viewport_start + available_width).min(graphemes.len());
        let visible = graphemes
            .get(viewport_start..end_pos)
            .unwrap_or(&[])
            .join("");
        spans.push(Span::styled(
            visible,
            Style::default().fg(self.config.theme.input_text.into()),
        ));

        let paragraph = Paragraph::new(Line::from(spans)).block(
            Block::default()
                .padding(Padding::new(3, 1, 1, 1))
                .borders(Borders::NONE)
                .style(Style::default().bg(self.config.theme.input_bg.into())),
        );

        let cursor_coord = if self.cursor.is_visible() {
            let visible_chars_before_cursor = if cursor_pos > viewport_start {
                let chars_before = graphemes.get(viewport_start..cursor_pos).unwrap_or(&[]);
                chars_before
                    .iter()
                    .map(|g| UnicodeWidthStr::width(*g))
                    .sum::<usize>()
            } else {
                0
            };

            let rel_x = (prompt_width + visible_chars_before_cursor) as u16;
            let rel_y = 0u16;

            Some((rel_x, rel_y))
        } else {
            None
        };

        (paragraph, cursor_coord)
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<String> {
        self.handle_key_event(key)
    }

    fn as_input_state(&mut self) -> Option<&mut dyn InputWidget> {
        Some(self)
    }

    fn get_backup_data(&self) -> Option<InputStateBackup> {
        Some(self.export_state())
    }

    fn restore_backup_data(&mut self, backup: InputStateBackup) {
        self.import_state(backup);
    }
}

impl InputWidget for InputState {
    fn update_cursor_blink(&mut self) {
        self.cursor.update_blink();
    }
}
