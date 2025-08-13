// =====================================================
// FILE: src/input/input.rs - COMPLETE FIXED (Single Widget Impl)
// =====================================================

use crate::commands::handler::CommandHandler;
use crate::commands::history::{
    HistoryAction, HistoryConfig, HistoryEvent, HistoryEventHandler, HistoryKeyboardHandler,
    HistoryManager,
};
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::ui::cursor::{CursorKind, CursorType, UiCursor};
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

        log::debug!(
            "✅ InputState cursor updated via central API: {}",
            self.cursor.debug_info()
        );
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
        self.waiting_for_restart_confirmation = false;
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

    fn handle_restart_confirmation(&mut self, action: KeyAction) -> Option<String> {
        match action {
            KeyAction::Submit => {
                self.waiting_for_restart_confirmation = false;
                let confirm_short = crate::i18n::get_translation("system.input.confirm.short", &[]);
                let cancel_short = crate::i18n::get_translation("system.input.cancel.short", &[]);
                match self.content.trim().to_lowercase().as_str() {
                    input if input == confirm_short.to_lowercase() => {
                        self.content.clear();
                        Some("__RESTART__".to_string())
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
            KeyAction::Submit => {
                // ✅ KORRIGIERTE DEBUG COMMANDS - alle im gleichen match Block
                match self.content.trim() {
                    "cursor-debug" => {
                        let debug_info = format!(
                            "🎯 CURSOR COLOR DEBUG:\n\
                            📊 Theme: {}\n\
                            🎨 Expected input_cursor_color: {}\n\
                            🎨 Actual cursor color: {}\n\
                            🎨 Actual fg color: {}\n\
                            🔍 Cursor details:\n\
                            {}",
                            self.config.current_theme_name,
                            self.config.theme.input_cursor_color.to_name(),
                            self.cursor.color.to_name(),
                            self.cursor.fg.to_name(),
                            self.cursor.debug_info()
                        );
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        Some(debug_info)
                    }

                    "theme-config-debug" => {
                        let debug_info = format!(
                            "🔍 COMPLETE THEME CONFIG DEBUG:\n\
                            📁 Current Theme: {}\n\
                            🎨 input_cursor_color: {} ⬅️ CONFIG VALUE\n\
                            🎨 input_cursor: {}\n\
                            🎨 input_cursor_prefix: '{}'\n\
                            🎨 output_cursor_color: {}\n\
                            🎨 output_cursor: {}\n\
                            \n🎯 ACTUAL CURSOR STATE:\n\
                            🎨 cursor.color: {} ⬅️ ACTUAL VALUE\n\
                            🎯 cursor.ctype: {:?}\n\
                            👁️ cursor.visible: {}",
                            self.config.current_theme_name,
                            self.config.theme.input_cursor_color.to_name(),
                            self.config.theme.input_cursor,
                            self.config.theme.input_cursor_prefix,
                            self.config.theme.output_cursor_color.to_name(),
                            self.config.theme.output_cursor,
                            self.cursor.color.to_name(),
                            self.cursor.ctype,
                            self.cursor.is_visible()
                        );
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        Some(debug_info)
                    }

                    "color-test" => {
                        let test_colors = vec![
                            "Red",
                            "Green",
                            "Blue",
                            "Yellow",
                            "Magenta",
                            "Cyan",
                            "LightRed",
                            "LightGreen",
                            "LightBlue",
                            "LightYellow",
                            "LightMagenta",
                            "LightCyan",
                            "White",
                            "Black",
                        ];

                        let mut results = String::from("🎨 COLOR CONVERSION TEST:\n");
                        for color_name in test_colors {
                            match crate::ui::color::AppColor::from_string(color_name) {
                                Ok(color) => {
                                    results.push_str(&format!(
                                        "✅ '{}' → '{}'\n",
                                        color_name,
                                        color.to_name()
                                    ));
                                }
                                Err(e) => {
                                    results
                                        .push_str(&format!("❌ '{}' → ERROR: {}\n", color_name, e));
                                }
                            }
                        }

                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        Some(results)
                    }

                    "full-debug" => {
                        let (_, cursor_pos) = self.render_with_cursor();
                        let debug_info = format!(
                            "🔍 FULL CURSOR DEBUG:\n\
                            🎨 Config Theme: '{}'\n\
                            📝 input_cursor: '{}'\n\
                            🎯 Parsed Type: {:?}\n\
                            🔤 Symbol: '{}'\n\
                            👁️ Is Visible: {}\n\
                            📍 Position: {}\n\
                            🖥️ Terminal Pos: {:?}\n\
                            🔧 Match Block: {}\n\
                            ⚡ Should Use Terminal: {}",
                            self.config.current_theme_name,
                            self.config.theme.input_cursor,
                            self.cursor.ctype,
                            self.cursor.get_symbol(),
                            self.cursor.is_visible(),
                            self.cursor.get_position(),
                            cursor_pos,
                            matches!(self.cursor.ctype, CursorType::Block),
                            !matches!(self.cursor.ctype, CursorType::Block)
                        );
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        Some(debug_info)
                    }

                    "term-test" => {
                        let info = format!(
                            "🖥️ TERMINAL INFO:\n\
                            📺 Terminal: {:?}\n\
                            🎯 Cursor Support: Testing...\n\
                            💡 Try: ESC[?25h (show cursor)\n\
                            💡 Or: Different terminal app",
                            std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
                        );
                        self.content.clear();
                        self.cursor.reset_for_empty_text();
                        Some(info)
                    }

                    // ✅ ALLE ANDEREN COMMANDS (nicht-debug)
                    _ => {
                        if self.content.is_empty() {
                            return None;
                        }
                        if self.validate_input(&self.content).is_ok() {
                            let content = std::mem::take(&mut self.content);
                            self.cursor.reset_for_empty_text();
                            self.history_manager.add_entry(content.clone());
                            let result = self.command_handler.handle_input(&content);

                            if let Some(event) =
                                HistoryEventHandler::handle_command_result(&result.message)
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
                                let feedback_text =
                                    if result.message.starts_with("__RESTART_FORCE__") {
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
        }
    }

    pub fn import_state(&mut self, backup: InputStateBackup) {
        self.content = backup.content;
        self.history_manager.import_entries(backup.history);
        self.cursor.update_text_length(&self.content);
        log::debug!(
            "✅ InputState imported: {} chars, {} history entries",
            self.content.len(),
            self.history_manager.entry_count()
        );
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_history_count(&self) -> usize {
        self.history_manager.entry_count()
    }

    // =====================================================
    // ✅ RENDERING METHODS (IN InputState impl, NICHT Widget trait!)
    // =====================================================

    /// ✅ FIXED: BLOCK-CURSOR mit korrekter Farbe
    fn render_block_cursor(
        &self,
        spans: &mut Vec<Span<'static>>,
        graphemes: &[&str],
        cursor_pos: usize,
        viewport_start: usize,
        available_width: usize,
    ) {
        // Text vor Cursor
        if cursor_pos > viewport_start {
            let visible_text = graphemes[viewport_start..cursor_pos].join("");
            if !visible_text.is_empty() {
                spans.push(Span::styled(
                    visible_text,
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        }

        // ✅ ZEICHEN AM CURSOR: Invertiert wenn sichtbar
        let char_at_cursor = graphemes.get(cursor_pos).copied().unwrap_or(" ");
        if self.cursor.is_visible() {
            // Invertierung: Farben tauschen
            spans.push(Span::styled(
                char_at_cursor.to_string(),
                Style::default()
                    .fg(self.config.theme.input_bg.into()) // Hintergrund wird Vordergrund
                    .bg(self.config.theme.input_cursor_color.into()), // ✅ FIXED: Richtige Cursor-Farbe
            ));
        } else {
            // Normal: Kein Cursor sichtbar
            spans.push(Span::styled(
                char_at_cursor.to_string(),
                Style::default().fg(self.config.theme.input_text.into()),
            ));
        }

        // Text nach Cursor
        let end_pos = (viewport_start + available_width).min(graphemes.len());
        if cursor_pos + 1 < end_pos {
            let remaining_text = graphemes[cursor_pos + 1..end_pos].join("");
            if !remaining_text.is_empty() {
                spans.push(Span::styled(
                    remaining_text,
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        }
    }

    /// ✅ NEUE METHODE: Symbol-Cursor (PIPE + UNDERSCORE)
    fn render_symbol_cursor(
        &self,
        spans: &mut Vec<Span<'static>>,
        graphemes: &[&str],
        cursor_pos: usize,
        viewport_start: usize,
        available_width: usize,
    ) {
        let end_pos = (viewport_start + available_width).min(graphemes.len());

        // Text VOR Cursor
        if cursor_pos > viewport_start {
            let visible_text = graphemes[viewport_start..cursor_pos].join("");
            if !visible_text.is_empty() {
                spans.push(Span::styled(
                    visible_text,
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        }

        // ✅ CURSOR-SYMBOL mit korrekter Farbe (wenn sichtbar)
        if self.cursor.is_visible() {
            let cursor_symbol = self.cursor.get_symbol(); // "|" oder "_"
            spans.push(Span::styled(
                cursor_symbol.to_string(),
                Style::default()
                    .fg(self.config.theme.input_cursor_color.into()) // ✅ Richtige Farbe!
                    .bg(self.config.theme.input_bg.into()),
            ));
        }

        // Text NACH Cursor
        if cursor_pos < end_pos {
            let remaining_text = graphemes[cursor_pos..end_pos].join("");
            if !remaining_text.is_empty() {
                spans.push(Span::styled(
                    remaining_text,
                    Style::default().fg(self.config.theme.input_text.into()),
                ));
            }
        }
    }
}

// ✅ SINGLE Widget Implementation - NO DUPLICATES!
impl Widget for InputState {
    fn render(&self) -> Paragraph {
        self.render_with_cursor().0
    }

    /// ✅ FIXED: PIPE-Cursor auch als eigenes Symbol rendern!
    fn render_with_cursor(&self) -> (Paragraph, Option<(u16, u16)>) {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();
        let mut spans = Vec::with_capacity(8);

        // Prompt-Text aus Theme
        let prompt_display = self.config.theme.input_cursor_prefix.clone();
        let prompt_width = prompt_display.graphemes(true).count();
        spans.push(Span::styled(
            prompt_display,
            Style::default().fg(self.config.theme.input_cursor_color.into()),
        ));

        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10
        } else {
            0
        };

        match self.cursor.ctype {
            CursorType::Block => {
                // BLOCK: Invertiere das Zeichen unter dem Cursor
                self.render_block_cursor(
                    &mut spans,
                    &graphemes,
                    cursor_pos,
                    viewport_start,
                    available_width,
                );
                let paragraph = Paragraph::new(Line::from(spans)).block(
                    Block::default()
                        .padding(Padding::new(3, 1, 1, 1))
                        .borders(Borders::NONE)
                        .style(Style::default().bg(self.config.theme.input_bg.into())),
                );
                (paragraph, None)
            }
            CursorType::Pipe | CursorType::Underscore => {
                // ✅ PIPE + UNDERSCORE: Beide als eigenes Symbol rendern!
                self.render_symbol_cursor(
                    &mut spans,
                    &graphemes,
                    cursor_pos,
                    viewport_start,
                    available_width,
                );
                let paragraph = Paragraph::new(Line::from(spans)).block(
                    Block::default()
                        .padding(Padding::new(3, 1, 1, 1))
                        .borders(Borders::NONE)
                        .style(Style::default().bg(self.config.theme.input_bg.into())),
                );
                (paragraph, None) // ✅ KEIN Terminal-Cursor mehr!
            }
        }
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
