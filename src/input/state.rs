// ## FILE: src/input/state.rs - KOMPRIMIERTE VERSION
use crate::commands::handler::CommandHandler;
use crate::commands::history::{
    HistoryAction, HistoryConfig, HistoryEvent, HistoryEventHandler, HistoryKeyboardHandler,
    HistoryManager,
};
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::ui::cursor::{CursorKind, UiCursor};
use crate::ui::widget::{AnimatedWidget, CursorWidget, StatefulWidget, Widget};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct InputState {
    content: String,
    cursor: UiCursor,
    prompt: String,
    history_manager: HistoryManager,
    config: Config,
    command_handler: CommandHandler,
    keyboard_manager: KeyboardManager,
    confirmation_state: ConfirmationState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfirmationState {
    None,
    Exit,
    Restart,
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
            confirmation_state: ConfirmationState::None,
        }
    }

    pub fn update_from_config(&mut self, config: &Config) {
        self.cursor.update_from_config(config);
        self.prompt = config.theme.input_cursor_prefix.clone();
        self.config = config.clone();
    }

    pub fn reset_for_language_change(&mut self) {
        self.confirmation_state = ConfirmationState::None;
        self.clear_input();
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
    pub fn get_history_count(&self) -> usize {
        self.history_manager.entry_count()
    }

    // ✅ KOMPRIMIERTE INPUT HANDLING
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<String> {
        // History navigation
        if let Some(action) = HistoryKeyboardHandler::get_history_action(&key) {
            return self.handle_history(action);
        }

        if key.code == KeyCode::Esc {
            return None;
        }

        let action = self.keyboard_manager.get_action(&key);

        // Confirmation handling
        if self.confirmation_state != ConfirmationState::None {
            return self.handle_confirmation(action);
        }

        // Regular input handling
        match action {
            KeyAction::Submit => self.handle_submit(),
            KeyAction::PasteBuffer => self.handle_paste(),
            KeyAction::CopySelection => self.handle_copy(),
            KeyAction::ClearLine => self.handle_clear_line(),
            KeyAction::InsertChar(c) => {
                self.insert_char(c);
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
                self.handle_backspace();
                None
            }
            KeyAction::Delete => {
                self.handle_delete();
                None
            }
            _ => None,
        }
    }

    // ✅ KOMPRIMIERTE CONFIRMATION LOGIC
    fn handle_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                let confirm = t!("system.input.confirm.short").to_lowercase();
                let result = if self.content.trim().to_lowercase() == confirm {
                    match self.confirmation_state {
                        ConfirmationState::Exit => "__EXIT__".to_string(),
                        ConfirmationState::Restart => "__RESTART__".to_string(),
                        _ => get_translation("system.input.cancelled", &[]),
                    }
                } else {
                    get_translation("system.input.cancelled", &[])
                };

                self.confirmation_state = ConfirmationState::None;
                self.clear_input();
                Some(result)
            }
            KeyAction::InsertChar(c) => {
                let confirm_char = t!("system.input.confirm.short").to_lowercase();
                let cancel_char = t!("system.input.cancel.short").to_lowercase();

                if [confirm_char, cancel_char].contains(&c.to_lowercase().to_string()) {
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

    fn handle_history(&mut self, action: HistoryAction) -> Option<String> {
        let entry = match action {
            HistoryAction::NavigatePrevious => self.history_manager.navigate_previous(),
            HistoryAction::NavigateNext => self.history_manager.navigate_next(),
        };

        if let Some(entry) = entry {
            self.content = entry;
            self.cursor.update_text_length(&self.content);
            self.cursor.move_to_end();
        }
        None
    }

    fn handle_submit(&mut self) -> Option<String> {
        if self.content.is_empty() || self.content.trim().is_empty() {
            return None;
        }

        if self.content.graphemes(true).count() > 1024 {
            return Some(get_translation("system.input.too_long", &["1024"]));
        }

        let content = std::mem::take(&mut self.content);
        self.cursor.reset_for_empty_text();
        self.history_manager.add_entry(content.clone());

        let result = self.command_handler.handle_input(&content);

        // Handle special responses (unchanged)
        if let Some(event) = HistoryEventHandler::handle_command_result(&result.message) {
            return Some(self.handle_history_event(event));
        }

        // Handle confirmations with i18n
        if result.message.starts_with("__CONFIRM_EXIT__") {
            self.confirmation_state = ConfirmationState::Exit;
            return Some(result.message.replace("__CONFIRM_EXIT__", ""));
        }
        if result.message.starts_with("__CONFIRM_RESTART__") {
            self.confirmation_state = ConfirmationState::Restart;
            return Some(result.message.replace("__CONFIRM_RESTART__", ""));
        }

        // Handle restart commands (unchanged)
        if result.message.starts_with("__RESTART") {
            let feedback = if result.message.starts_with("__RESTART_FORCE__") {
                result
                    .message
                    .replace("__RESTART_FORCE__", "")
                    .trim()
                    .to_string()
            } else {
                result.message.replace("__RESTART__", "").trim().to_string()
            };

            return Some(if feedback.is_empty() {
                "__RESTART__".to_string()
            } else {
                format!("__RESTART_WITH_MSG__{}", feedback)
            });
        }

        if result.should_exit {
            Some(format!("__EXIT__{}", result.message))
        } else {
            Some(result.message)
        }
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

    // ✅ KOMPRIMIERTE CLIPBOARD OPERATIONS
    fn handle_paste(&mut self) -> Option<String> {
        let text = self.read_clipboard()?;
        let clean = text
            .replace(['\n', '\r', '\t'], " ")
            .chars()
            .filter(|c| !c.is_control() || *c == ' ')
            .collect::<String>();

        if clean.is_empty() {
            return Some(get_translation("system.input.clipboard.empty", &[]));
        }

        let current_len = self.content.graphemes(true).count();
        let available = self.config.input_max_length.saturating_sub(current_len);
        let paste_text = clean.graphemes(true).take(available).collect::<String>();

        if !paste_text.is_empty() {
            let byte_pos = self.cursor.get_byte_position(&self.content);
            self.content.insert_str(byte_pos, &paste_text);
            let chars_added = paste_text.graphemes(true).count();
            self.cursor.update_text_length(&self.content);

            for _ in 0..chars_added {
                self.cursor.move_right();
            }
            Some(get_translation(
                "system.input.clipboard.pasted",
                &[&chars_added.to_string()],
            ))
        } else {
            Some(get_translation(
                "system.input.clipboard.nothing_to_paste",
                &[],
            ))
        }
    }

    fn handle_copy(&self) -> Option<String> {
        if self.content.is_empty() {
            return Some(get_translation(
                "system.input.clipboard.nothing_to_copy",
                &[],
            ));
        }

        if self.write_clipboard(&self.content) {
            let preview = if self.content.chars().count() > 50 {
                format!("{}...", self.content.chars().take(50).collect::<String>())
            } else {
                self.content.clone()
            };
            Some(get_translation(
                "system.input.clipboard.copied",
                &[&preview],
            ))
        } else {
            Some(get_translation("system.input.clipboard.copy_failed", &[]))
        }
    }

    fn handle_clear_line(&mut self) -> Option<String> {
        if self.content.is_empty() {
            return None;
        }

        let result = if self.write_clipboard(&self.content) {
            let preview = if self.content.chars().count() > 50 {
                format!("{}...", self.content.chars().take(50).collect::<String>())
            } else {
                self.content.clone()
            };
            get_translation("system.input.clipboard.cut", &[&preview])
        } else {
            get_translation("system.input.clipboard.cleared", &[])
        };

        self.clear_input();
        Some(result)
    }

    // ✅ KOMPRIMIERTE CLIPBOARD SYSTEM
    fn read_clipboard(&self) -> Option<String> {
        let output = self.get_clipboard_cmd("read")?.output().ok()?;
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn write_clipboard(&self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        if let Some(mut cmd) = self.get_clipboard_cmd("write") {
            if let Ok(mut child) = cmd.stdin(std::process::Stdio::piped()).spawn() {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().is_ok();
            }
        }
        false
    }

    fn get_clipboard_cmd(&self, op: &str) -> Option<std::process::Command> {
        #[cfg(target_os = "macos")]
        {
            Some(std::process::Command::new(if op == "read" {
                "pbpaste"
            } else {
                "pbcopy"
            }))
        }

        #[cfg(target_os = "linux")]
        {
            let mut cmd = std::process::Command::new("xclip");
            if op == "read" {
                cmd.args(["-selection", "clipboard", "-o"]);
            } else {
                cmd.args(["-selection", "clipboard"]);
            }
            Some(cmd)
        }

        #[cfg(target_os = "windows")]
        {
            if op == "read" {
                let mut cmd = std::process::Command::new("powershell");
                cmd.args(["-Command", "Get-Clipboard"]);
                Some(cmd)
            } else {
                None // Windows write handling unterschiedlich
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        None
    }

    // ✅ KOMPRIMIERTE TEXT EDITING
    fn insert_char(&mut self, c: char) {
        if self.content.graphemes(true).count() < self.config.input_max_length {
            let byte_pos = self.cursor.get_byte_position(&self.content);
            self.content.insert(byte_pos, c);
            self.cursor.update_text_length(&self.content);
            self.cursor.move_right();
        }
    }

    fn handle_backspace(&mut self) {
        if self.content.is_empty() || self.cursor.get_position() == 0 {
            return;
        }

        let current = self.cursor.get_byte_position(&self.content);
        let prev = self.cursor.get_prev_byte_position(&self.content);

        if prev < current && current <= self.content.len() {
            self.cursor.move_left();
            self.content.replace_range(prev..current, "");
            self.cursor.update_text_length(&self.content);

            if self.content.is_empty() {
                self.cursor.reset_for_empty_text();
            }
        }
    }

    fn handle_delete(&mut self) {
        let text_len = self.content.graphemes(true).count();
        if self.cursor.get_position() >= text_len || text_len == 0 {
            return;
        }

        let current = self.cursor.get_byte_position(&self.content);
        let next = self.cursor.get_next_byte_position(&self.content);

        if current < next && next <= self.content.len() {
            self.content.replace_range(current..next, "");
            self.cursor.update_text_length(&self.content);

            if self.content.is_empty() {
                self.cursor.reset_for_empty_text();
            }
        }
    }

    fn clear_input(&mut self) {
        self.content.clear();
        self.history_manager.reset_position();
        self.cursor.move_to_start();
    }
}

// ✅ WIDGET TRAIT IMPLEMENTATIONS (angepasst an neue Namen)
impl Widget for InputState {
    fn render(&self) -> Paragraph {
        self.render_with_cursor().0
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<String> {
        self.handle_key_event(key)
    }
}

impl CursorWidget for InputState {
    fn render_with_cursor(&self) -> (Paragraph, Option<(u16, u16)>) {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();
        let prompt_width = self.prompt.width();
        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        // Viewport calculation
        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10
        } else {
            0
        };

        // Create spans
        let mut spans = vec![Span::styled(
            &self.prompt,
            Style::default().fg(self.config.theme.input_cursor_color.into()),
        )];

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

        // Cursor coordinates
        let cursor_coord = if self.cursor.is_visible() && cursor_pos >= viewport_start {
            let chars_before = graphemes.get(viewport_start..cursor_pos).unwrap_or(&[]);
            let visible_width: usize = chars_before
                .iter()
                .map(|g| UnicodeWidthStr::width(*g))
                .sum();
            Some(((prompt_width + visible_width) as u16, 0u16))
        } else {
            None
        };

        (paragraph, cursor_coord)
    }
}

impl StatefulWidget for InputState {
    fn export_state(&self) -> InputStateBackup {
        InputStateBackup {
            content: self.content.clone(),
            history: self.history_manager.get_all_entries(),
            cursor_pos: self.cursor.get_current_position(),
        }
    }

    fn import_state(&mut self, state: InputStateBackup) {
        self.content = state.content;
        self.history_manager.import_entries(state.history);
        self.cursor.update_text_length(&self.content);
    }
}

impl AnimatedWidget for InputState {
    fn tick(&mut self) {
        self.cursor.update_blink();
    }
}
