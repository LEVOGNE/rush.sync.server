// =====================================================
// FILE: src/input/input.rs - COMPLETE FIXED mit t! Makro Support
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
use std::process::Command;
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
        log::debug!("InputState reset for language change");
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

    // ✅ ECHTES CLIPBOARD LESEN (Mac/Linux/Windows)
    fn read_clipboard(&self) -> Option<String> {
        #[cfg(target_os = "macos")]
        {
            // Mac: pbpaste verwenden
            match Command::new("pbpaste").output() {
                Ok(output) => {
                    let clipboard_text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !clipboard_text.trim().is_empty() {
                        log::info!("📋 Mac clipboard read: {} chars", clipboard_text.len());
                        Some(clipboard_text.trim_end_matches('\n').to_string())
                    } else {
                        log::debug!("📋 Mac clipboard empty");
                        None
                    }
                }
                Err(e) => {
                    log::error!("📋 Failed to read Mac clipboard: {}", e);
                    None
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: xclip oder xsel verwenden
            match Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
            {
                Ok(output) => {
                    let clipboard_text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !clipboard_text.trim().is_empty() {
                        log::info!("📋 Linux clipboard read: {} chars", clipboard_text.len());
                        Some(clipboard_text.trim_end_matches('\n').to_string())
                    } else {
                        None
                    }
                }
                Err(_) => {
                    // Fallback zu xsel
                    match Command::new("xsel").args(["-b", "-o"]).output() {
                        Ok(output) => {
                            let clipboard_text =
                                String::from_utf8_lossy(&output.stdout).to_string();
                            Some(clipboard_text.trim_end_matches('\n').to_string())
                        }
                        Err(e) => {
                            log::error!("📋 Failed to read Linux clipboard: {}", e);
                            None
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: PowerShell verwenden
            match Command::new("powershell")
                .args(["-Command", "Get-Clipboard"])
                .output()
            {
                Ok(output) => {
                    let clipboard_text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !clipboard_text.trim().is_empty() {
                        log::info!("📋 Windows clipboard read: {} chars", clipboard_text.len());
                        Some(clipboard_text.trim_end_matches('\n').to_string())
                    } else {
                        None
                    }
                }
                Err(e) => {
                    log::error!("📋 Failed to read Windows clipboard: {}", e);
                    None
                }
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            log::warn!("📋 Clipboard not supported on this platform");
            None
        }
    }

    // ✅ ECHTES CLIPBOARD SCHREIBEN
    fn write_clipboard(&self, text: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            match Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait().map(|_| ())
                }) {
                Ok(_) => {
                    log::info!("📋 Mac clipboard written: {} chars", text.len());
                    true
                }
                Err(e) => {
                    log::error!("📋 Failed to write Mac clipboard: {}", e);
                    false
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            match Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait().map(|_| ())
                }) {
                Ok(_) => {
                    log::info!("📋 Linux clipboard written: {} chars", text.len());
                    true
                }
                Err(e) => {
                    log::error!("📋 Failed to write Linux clipboard: {}", e);
                    false
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            match Command::new("powershell")
                .args(["-Command", &format!("'{}' | Set-Clipboard", text)])
                .output()
            {
                Ok(_) => {
                    log::info!("📋 Windows clipboard written: {} chars", text.len());
                    true
                }
                Err(e) => {
                    log::error!("📋 Failed to write Windows clipboard: {}", e);
                    false
                }
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            log::warn!("📋 Clipboard write not supported on this platform");
            false
        }
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

        // KOMPLETTER match action Block in handle_key_event:

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
                    📍 Cursor details:\n\
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
                    📝 Current Theme: {}\n\
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
                    📍 input_cursor: '{}'\n\
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

            // ✅ EINMALIGER ClearLine Handler
            KeyAction::ClearLine => {
                if !self.content.is_empty() {
                    log::info!("🧹 Clearing input line ({} chars)", self.content.len());
                    self.content.clear();
                    self.cursor.reset_for_empty_text();
                    self.history_manager.reset_position();
                    Some("Input cleared".to_string())
                } else {
                    log::debug!("🧹 ClearLine called but input already empty");
                    None
                }
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

            // ✅ COPY/PASTE IMPLEMENTATION
            KeyAction::CopySelection => {
                if !self.content.is_empty() {
                    if self.write_clipboard(&self.content) {
                        log::info!("📋 Copied to clipboard: '{}'", self.content);
                        Some(format!("📋 Copied: {}", self.content))
                    } else {
                        log::error!("📋 Failed to copy to clipboard");
                        Some("❌ Copy failed".to_string())
                    }
                } else {
                    log::debug!("📋 Copy called but nothing to copy");
                    Some("❌ Nothing to copy".to_string())
                }
            }

            // ✅ ECHTE PASTE-IMPLEMENTATION
            KeyAction::PasteBuffer => {
                log::debug!("📋 Paste requested");

                if let Some(clipboard_text) = self.read_clipboard() {
                    // Clipboard-Text validieren und einfügen
                    let sanitized = clipboard_text
                        .replace('\n', " ") // Newlines zu Spaces
                        .replace('\r', "") // Carriage returns entfernen
                        .chars()
                        .filter(|c| !c.is_control() || *c == ' ') // Nur printable chars + spaces
                        .collect::<String>();

                    if !sanitized.is_empty() {
                        // Check gegen max_length
                        let available_space = self
                            .config
                            .input_max_length
                            .saturating_sub(self.content.graphemes(true).count());

                        let paste_text = if sanitized.graphemes(true).count() > available_space {
                            // Kürzen wenn zu lang
                            sanitized
                                .graphemes(true)
                                .take(available_space)
                                .collect::<String>()
                        } else {
                            sanitized
                        };

                        if !paste_text.is_empty() {
                            // An Cursor-Position einfügen
                            let byte_pos = self.cursor.get_byte_position(&self.content);
                            self.content.insert_str(byte_pos, &paste_text);

                            // Cursor entsprechend bewegen
                            let chars_added = paste_text.graphemes(true).count();
                            self.cursor.update_text_length(&self.content);
                            for _ in 0..chars_added {
                                self.cursor.move_right();
                            }

                            log::info!("📋 Pasted {} chars at position {}", chars_added, byte_pos);
                            Some(format!("📋 Pasted: {} chars", chars_added))
                        } else {
                            Some("❌ Nothing to paste (text too long)".to_string())
                        }
                    } else {
                        Some("❌ Clipboard contains no valid text".to_string())
                    }
                } else {
                    Some("❌ Clipboard empty or inaccessible".to_string())
                }
            }

            // Alle anderen Actions als NoAction behandeln
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
    // ✅ 2-LAYER RENDERING: Text separat, Cursor als Terminal-Cursor
    // =====================================================
}

// ✅ SINGLE Widget Implementation - NO DUPLICATES!
impl Widget for InputState {
    fn render(&self) -> Paragraph {
        self.render_with_cursor().0
    }

    /// ✅ 2-LAYER CURSOR: Text + Terminal-Cursor getrennt!
    fn render_with_cursor(&self) -> (Paragraph, Option<(u16, u16)>) {
        use unicode_width::UnicodeWidthStr;

        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let cursor_pos = self.cursor.get_position();

        let prompt_display = self.config.theme.input_cursor_prefix.clone();
        let prompt_width = prompt_display.width(); // ✅ visuelle Zellenbreite!

        let available_width = self
            .config
            .input_max_length
            .saturating_sub(prompt_width + 4);

        let viewport_start = if cursor_pos > available_width {
            cursor_pos - available_width + 10
        } else {
            0
        };

        // ✅ LAYER 1: KOMPLETTER TEXT (inklusive Zeichen an Cursor-Position!)
        let mut spans = Vec::new();
        spans.push(Span::styled(
            prompt_display,
            Style::default().fg(self.config.theme.input_cursor_color.into()),
        ));

        // ✅ WICHTIG: GANZEN sichtbaren Text rendern (auch an Cursor-Position)
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

        // ✅ LAYER 2: Cursor-Koordinate berechnen (OVERLAY über existierendem Text!)
        let cursor_coord = if self.cursor.is_visible() {
            // ✅ CRITICAL FIX: Cursor ÜBER das Zeichen an cursor_pos legen!
            let visible_chars_before_cursor = if cursor_pos > viewport_start {
                // Nur Zeichen VOR dem Cursor zählen (nicht bis cursor_pos!)
                let chars_before = graphemes.get(viewport_start..cursor_pos).unwrap_or(&[]);
                chars_before
                    .iter()
                    .map(|g| UnicodeWidthStr::width(*g))
                    .sum::<usize>()
            } else {
                0
            };

            // ✅ WICHTIG: rel_x zeigt GENAU auf das Zeichen, wo der Cursor stehen soll
            let rel_x = (prompt_width + visible_chars_before_cursor) as u16;
            let rel_y = 0u16;

            log::debug!(
                "🎯 CURSOR OVERLAY: cursor_pos={}, viewport_start={}, chars_before={}, rel_x={}, prompt_width={}",
                cursor_pos, viewport_start, visible_chars_before_cursor, rel_x, prompt_width
            );

            Some((rel_x, rel_y))
        } else {
            None // Cursor unsichtbar (Blinken)
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
