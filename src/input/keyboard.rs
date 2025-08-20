use crate::core::constants::DOUBLE_ESC_THRESHOLD;
use crate::core::prelude::*;
use crossterm::event::KeyModifiers;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    MoveLeft,
    MoveRight,
    MoveToStart,
    MoveToEnd,
    InsertChar(char),
    Backspace,
    Delete,
    Submit,
    Cancel,
    Quit,
    ClearLine,
    CopySelection,
    PasteBuffer,
    NoAction,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
}

lazy_static! {
    static ref LAST_ESC_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
    static ref ESCAPE_SEQUENCE_BUFFER: Mutex<Vec<char>> = Mutex::new(Vec::new());
}

pub struct KeyboardManager {
    double_press_threshold: Duration,
    sequence_timeout: Duration,
    last_key_time: Instant,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            double_press_threshold: Duration::from_millis(DOUBLE_ESC_THRESHOLD),
            sequence_timeout: Duration::from_millis(100),
            last_key_time: Instant::now(),
        }
    }

    // Consolidated security filtering
    fn is_safe_char(&mut self, c: char) -> bool {
        // Filter dangerous control chars and sequences
        if matches!(c, '\x00'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F') {
            return false;
        }

        // Filter suspicious non-ASCII chars (except common European chars)
        if !c.is_ascii() && !c.is_alphabetic() && !"äöüßÄÖÜ€".contains(c) {
            return false;
        }

        // Check for terminal sequence patterns
        !self.detect_terminal_sequence(c)
    }

    fn detect_terminal_sequence(&mut self, c: char) -> bool {
        let now = Instant::now();

        // Reset old buffer
        if now.duration_since(self.last_key_time) > self.sequence_timeout {
            if let Ok(mut buffer) = ESCAPE_SEQUENCE_BUFFER.lock() {
                buffer.clear();
            }
        }
        self.last_key_time = now;

        // Check sequences
        if let Ok(mut buffer) = ESCAPE_SEQUENCE_BUFFER.lock() {
            buffer.push(c);
            let sequence: String = buffer.iter().collect();

            // Detect dangerous patterns
            let is_dangerous = sequence.to_lowercase().contains("tmux")
                || (sequence.len() > 3
                    && sequence.chars().all(|ch| ch.is_ascii_digit() || ch == ';'))
                || sequence.contains("///")
                || sequence.contains(";;;");

            if is_dangerous {
                buffer.clear();
            }
            if buffer.len() > 20 {
                buffer.drain(0..10);
            }

            is_dangerous
        } else {
            false
        }
    }

    pub fn get_action(&mut self, key: &KeyEvent) -> KeyAction {
        // Handle ESC double-press
        if key.code == KeyCode::Esc {
            return self.handle_escape();
        }

        // Filter dangerous characters
        if let KeyCode::Char(c) = key.code {
            if !self.is_safe_char(c) {
                return KeyAction::NoAction;
            }
        }

        // Quick scroll detection
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Up => return KeyAction::ScrollUp,
                KeyCode::Down => return KeyAction::ScrollDown,
                _ => {}
            }
        }

        // Main key mapping - consolidated
        match (key.code, key.modifiers) {
            // Basic movement
            (KeyCode::Left, KeyModifiers::NONE) => KeyAction::MoveLeft,
            (KeyCode::Right, KeyModifiers::NONE) => KeyAction::MoveRight,
            (KeyCode::Home, KeyModifiers::NONE) => KeyAction::MoveToStart,
            (KeyCode::End, KeyModifiers::NONE) => KeyAction::MoveToEnd,
            (KeyCode::Enter, KeyModifiers::NONE) => KeyAction::Submit,

            // Scrolling
            (KeyCode::PageUp, KeyModifiers::NONE) => KeyAction::PageUp,
            (KeyCode::PageDown, KeyModifiers::NONE) => KeyAction::PageDown,

            // Text editing
            (KeyCode::Backspace, KeyModifiers::NONE) => KeyAction::Backspace,
            (KeyCode::Delete, KeyModifiers::NONE) => KeyAction::Delete,

            // Platform-specific shortcuts - consolidated
            (KeyCode::Char(c), mods) => self.handle_char_with_modifiers(c, mods),

            // Arrow keys with modifiers
            (KeyCode::Left, mods) if self.is_move_modifier(mods) => KeyAction::MoveToStart,
            (KeyCode::Right, mods) if self.is_move_modifier(mods) => KeyAction::MoveToEnd,

            // Backspace with modifiers
            (KeyCode::Backspace, mods) if self.is_clear_modifier(mods) => KeyAction::ClearLine,

            _ => KeyAction::NoAction,
        }
    }

    fn handle_escape(&self) -> KeyAction {
        let now = Instant::now();
        let mut last_press = LAST_ESC_PRESS.lock().unwrap_or_else(|p| p.into_inner());

        if let Some(prev_press) = *last_press {
            if now.duration_since(prev_press) <= self.double_press_threshold {
                *last_press = None;
                return KeyAction::Quit;
            }
        }

        *last_press = Some(now);
        KeyAction::NoAction
    }

    fn handle_char_with_modifiers(&self, c: char, mods: KeyModifiers) -> KeyAction {
        // Safe character input (no modifiers or just shift)
        if mods.is_empty() || mods == KeyModifiers::SHIFT {
            return if c.is_ascii_control() && c != '\t' {
                KeyAction::NoAction
            } else {
                KeyAction::InsertChar(c)
            };
        }

        // Shortcut handling - consolidated for all platforms
        match c {
            'c' if self.is_copy_modifier(mods) => KeyAction::CopySelection,
            'v' if self.is_paste_modifier(mods) => KeyAction::PasteBuffer,
            'x' if self.is_cut_modifier(mods) => KeyAction::ClearLine,
            'a' if self.is_select_modifier(mods) => KeyAction::MoveToStart,
            'e' if self.is_end_modifier(mods) => KeyAction::MoveToEnd,
            'u' if self.is_clear_modifier(mods) => KeyAction::ClearLine,
            _ => KeyAction::NoAction,
        }
    }

    // Platform-agnostic modifier checks
    fn is_copy_modifier(&self, mods: KeyModifiers) -> bool {
        mods.contains(KeyModifiers::SUPER)
            || mods.contains(KeyModifiers::CONTROL)
            || mods.contains(KeyModifiers::ALT)
    }

    fn is_paste_modifier(&self, mods: KeyModifiers) -> bool {
        self.is_copy_modifier(mods)
    }
    fn is_cut_modifier(&self, mods: KeyModifiers) -> bool {
        self.is_copy_modifier(mods)
    }
    fn is_select_modifier(&self, mods: KeyModifiers) -> bool {
        self.is_copy_modifier(mods)
    }
    fn is_end_modifier(&self, mods: KeyModifiers) -> bool {
        mods.contains(KeyModifiers::CONTROL) || mods.contains(KeyModifiers::ALT)
    }
    fn is_clear_modifier(&self, mods: KeyModifiers) -> bool {
        self.is_copy_modifier(mods)
    }
    fn is_move_modifier(&self, mods: KeyModifiers) -> bool {
        self.is_copy_modifier(mods)
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_escape_sequence_filtering() {
        let mut manager = KeyboardManager::new();

        // Test dangerous control character
        let ctrl_char = KeyEvent::new(KeyCode::Char('\x1B'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&ctrl_char), KeyAction::NoAction);

        // Test safe character
        let normal_char = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&normal_char), KeyAction::InsertChar('a'));
    }

    #[test]
    fn test_platform_shortcuts() {
        let mut manager = KeyboardManager::new();

        // Test CMD shortcuts (Mac)
        let cmd_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_c), KeyAction::CopySelection);

        // Test CTRL shortcuts (Windows/Linux)
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_c), KeyAction::CopySelection);

        // Test ALT shortcuts (fallback)
        let alt_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT);
        assert_eq!(manager.get_action(&alt_c), KeyAction::CopySelection);
    }

    #[test]
    fn test_scroll_actions() {
        let mut manager = KeyboardManager::new();

        let shift_up = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
        assert_eq!(manager.get_action(&shift_up), KeyAction::ScrollUp);

        let shift_down = KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT);
        assert_eq!(manager.get_action(&shift_down), KeyAction::ScrollDown);
    }

    #[test]
    fn test_double_escape() {
        let mut manager = KeyboardManager::new(); // ✅ FIX: mut hinzugefügt
        let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // First ESC should return NoAction
        assert_eq!(manager.get_action(&esc_key), KeyAction::NoAction);

        // Quick second ESC should return Quit (if within threshold)
        // Note: This test is simplified - in real usage, timing matters
    }
}
