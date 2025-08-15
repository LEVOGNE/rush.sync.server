// =====================================================
// FILE: src/input/keyboard.rs - MIT ESCAPE-CODE FILTER
// =====================================================

use crate::core::constants::DOUBLE_ESC_THRESHOLD;
use crate::core::prelude::*;
use crossterm::event::KeyModifiers;

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

use lazy_static::lazy_static;
use std::sync::Mutex;

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
        let manager = Self {
            double_press_threshold: Duration::from_millis(DOUBLE_ESC_THRESHOLD),
            sequence_timeout: Duration::from_millis(100), // 100ms timeout für Sequenzen
            last_key_time: Instant::now(),
        };

        manager.log_system_info();
        manager
    }

    fn log_system_info(&self) {
        log::info!("🔍 SYSTEM DEBUG INFO:");
        log::info!("   OS: {}", std::env::consts::OS);
        log::info!("   Arch: {}", std::env::consts::ARCH);

        if let Ok(term) = std::env::var("TERM") {
            log::info!("   Terminal: {}", term);
        }

        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            log::info!("   Terminal Program: {}", term_program);
        }

        #[cfg(target_os = "macos")]
        {
            log::info!("🍎 MAC DETECTED - CMD-Taste sollte als SUPER erkannt werden");
            log::info!("🛡️ Escape sequence filtering enabled");
        }

        log::info!("🔍 Drücke jetzt CMD+C um zu testen...");
    }

    /// 🛡️ NEUE FUNKTION: Filter Terminal Escape Sequences
    fn is_escape_sequence_char(&self, c: char) -> bool {
        // Filter bekannte problematische Zeichen/Sequenzen
        match c {
            // Control characters (außer Tab, Enter, etc.) - ESC ist schon in \x1B enthalten
            '\x00'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F' => {
                if c == '\x1B' {
                    log::warn!("🛡️ Blocked escape character");
                } else {
                    log::warn!("🛡️ Blocked control character: {:?}", c);
                }
                true
            }
            // Problematische Zeichen die in Terminal-Antworten vorkommen
            c if !c.is_ascii() && !c.is_alphabetic() && !"äöüßÄÖÜ€".contains(c) => {
                log::warn!("🛡️ Blocked suspicious character: {:?}", c);
                true
            }
            _ => false,
        }
    }

    /// 🛡️ NEUE FUNKTION: Detect sequence patterns
    fn detect_terminal_sequence(&mut self, c: char) -> bool {
        let now = Instant::now();

        // Reset buffer if too old
        if now.duration_since(self.last_key_time) > self.sequence_timeout {
            if let Ok(mut buffer) = ESCAPE_SEQUENCE_BUFFER.lock() {
                buffer.clear();
            }
        }

        self.last_key_time = now;

        // Add to buffer
        if let Ok(mut buffer) = ESCAPE_SEQUENCE_BUFFER.lock() {
            buffer.push(c);

            // Check for known bad sequences
            let sequence: String = buffer.iter().collect();

            // Pattern 1: "tmux" in any form
            if sequence.to_lowercase().contains("tmux") {
                log::warn!("🛡️ BLOCKED TMUX SEQUENCE: '{}'", sequence.escape_debug());
                buffer.clear();
                return true;
            }

            // Pattern 2: Long sequences of digits/semicolons
            if sequence.len() > 3 && sequence.chars().all(|ch| ch.is_ascii_digit() || ch == ';') {
                log::warn!("🛡️ BLOCKED DIGIT SEQUENCE: '{}'", sequence);
                buffer.clear();
                return true;
            }

            // Pattern 3: Control sequence indicators
            if sequence.contains("///") || sequence.contains(";;;") {
                log::warn!("🛡️ BLOCKED CONTROL SEQUENCE: '{}'", sequence);
                buffer.clear();
                return true;
            }

            // Limit buffer size
            if buffer.len() > 20 {
                buffer.drain(0..10); // Keep last 10 chars
            }
        }

        false
    }

    pub fn get_action(&mut self, key: &KeyEvent) -> KeyAction {
        // 🛡️ ERSTE VERTEIDIGUNG: Debug und Filter
        self.debug_key_event(key);

        // 🛡️ ZWEITE VERTEIDIGUNG: Char-Level filtering
        if let KeyCode::Char(c) = key.code {
            // Prüfe auf Escape-Sequenz-Zeichen
            if self.is_escape_sequence_char(c) {
                return KeyAction::NoAction;
            }

            // Prüfe auf Terminal-Sequenz-Pattern
            if self.detect_terminal_sequence(c) {
                return KeyAction::NoAction;
            }
        }

        // ESC Behandlung
        if key.code == KeyCode::Esc {
            let now = Instant::now();
            let mut last_press = LAST_ESC_PRESS.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });

            if let Some(prev_press) = *last_press {
                if now.duration_since(prev_press) <= self.double_press_threshold {
                    *last_press = None;
                    log::info!("🚪 Double ESC detected - Quit requested");
                    return KeyAction::Quit;
                }
            }

            *last_press = Some(now);
            return KeyAction::NoAction;
        }

        match (key.code, key.modifiers) {
            // ========== BEWEGUNG ==========
            (KeyCode::Left, KeyModifiers::NONE) => KeyAction::MoveLeft,
            (KeyCode::Right, KeyModifiers::NONE) => KeyAction::MoveRight,
            (KeyCode::Home, KeyModifiers::NONE) => KeyAction::MoveToStart,
            (KeyCode::End, KeyModifiers::NONE) => KeyAction::MoveToEnd,

            // ========== SUBMIT ==========
            (KeyCode::Enter, KeyModifiers::NONE) => KeyAction::Submit,

            // ========== SCROLLING ==========
            (KeyCode::Up, KeyModifiers::SHIFT) => KeyAction::ScrollUp,
            (KeyCode::Down, KeyModifiers::SHIFT) => KeyAction::ScrollDown,
            (KeyCode::PageUp, KeyModifiers::NONE) => KeyAction::PageUp,
            (KeyCode::PageDown, KeyModifiers::NONE) => KeyAction::PageDown,

            // ========== TEXT-BEARBEITUNG ==========
            (KeyCode::Backspace, KeyModifiers::NONE) => KeyAction::Backspace,
            (KeyCode::Delete, KeyModifiers::NONE) => KeyAction::Delete,

            // ========== 🍎 MAC: CMD-SHORTCUTS (SUPER) ==========
            (KeyCode::Char('c'), KeyModifiers::SUPER) => {
                log::info!("✅ Mac Cmd+C ERFOLGREICH erkannt → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::SUPER) => {
                log::info!("✅ Mac Cmd+V ERFOLGREICH erkannt → Paste");
                KeyAction::PasteBuffer
            }
            (KeyCode::Char('x'), KeyModifiers::SUPER) => {
                log::info!("✅ Mac Cmd+X ERFOLGREICH erkannt → Cut");
                KeyAction::ClearLine
            }
            (KeyCode::Char('a'), KeyModifiers::SUPER) => {
                log::info!("✅ Mac Cmd+A ERFOLGREICH erkannt → Select All");
                KeyAction::MoveToStart
            }

            // ========== 🍎 MAC: ALT/OPTION-SHORTCUTS ==========
            (KeyCode::Char('a'), KeyModifiers::ALT) => KeyAction::MoveToStart,
            (KeyCode::Char('e'), KeyModifiers::ALT) => KeyAction::MoveToEnd,
            (KeyCode::Char('u'), KeyModifiers::ALT) => KeyAction::ClearLine,
            (KeyCode::Char('c'), KeyModifiers::ALT) => {
                log::info!("🍎 Mac Alt+C detected → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::ALT) => {
                log::info!("🍎 Mac Alt+V detected → Paste");
                KeyAction::PasteBuffer
            }

            // ========== 🖥️ WINDOWS/LINUX: CTRL-SHORTCUTS ==========
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => KeyAction::MoveToStart,
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => KeyAction::MoveToEnd,
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => KeyAction::ClearLine,
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                log::info!("🖥️ Ctrl+C detected → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                log::info!("🖥️ Ctrl+V detected → Paste");
                KeyAction::PasteBuffer
            }

            // ========== BACKSPACE-KOMBINATIONEN ==========
            (KeyCode::Backspace, KeyModifiers::SUPER) => KeyAction::ClearLine,
            (KeyCode::Backspace, KeyModifiers::ALT) => KeyAction::ClearLine,
            (KeyCode::Backspace, KeyModifiers::CONTROL) => KeyAction::ClearLine,

            // ========== PFEILTASTEN MIT MODIFIERS ==========
            (KeyCode::Left, KeyModifiers::SUPER) => KeyAction::MoveToStart,
            (KeyCode::Right, KeyModifiers::SUPER) => KeyAction::MoveToEnd,
            (KeyCode::Left, KeyModifiers::CONTROL) => KeyAction::MoveToStart,
            (KeyCode::Right, KeyModifiers::CONTROL) => KeyAction::MoveToEnd,
            (KeyCode::Left, KeyModifiers::ALT) => KeyAction::MoveToStart,
            (KeyCode::Right, KeyModifiers::ALT) => KeyAction::MoveToEnd,

            // ========== 🛡️ SICHERE ZEICHEN-EINGABE ==========
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                // Extra check für normale Zeichen
                if c.is_ascii_control() && c != '\t' {
                    log::warn!("🛡️ Blocked control char in normal input: {:?}", c);
                    KeyAction::NoAction
                } else {
                    KeyAction::InsertChar(c)
                }
            }
            (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                // Extra check für Shift-Zeichen
                if c.is_ascii_control() {
                    log::warn!("🛡️ Blocked control char in shift input: {:?}", c);
                    KeyAction::NoAction
                } else {
                    KeyAction::InsertChar(c)
                }
            }

            // ========== FALLBACK ==========
            (_code, _modifiers) => {
                log::warn!(
                    "❓ UNBEKANNTE KEY-KOMBINATION: {:?} + {:?}",
                    _code,
                    _modifiers
                );
                KeyAction::NoAction
            }
        }
    }

    // ✅ DETAILLIERTER KEY-EVENT DEBUGGER
    fn debug_key_event(&self, key: &KeyEvent) {
        let modifier_debug = self.format_modifiers(key.modifiers);
        let key_debug = self.format_key_code(key.code);

        // 🛡️ SPECIAL: Check for suspicious patterns
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_control() || !c.is_ascii() {
                log::warn!(
                    "🛡️ SUSPICIOUS KEY: {} + {} (char: {:?} = U+{:04X})",
                    key_debug,
                    modifier_debug,
                    c,
                    c as u32
                );
            } else {
                log::info!("🔍 KEY EVENT: {} + {}", key_debug, modifier_debug);
            }
        } else {
            log::info!("🔍 KEY EVENT: {} + {}", key_debug, modifier_debug);
        }

        // CMD-Debugging
        if key.modifiers.contains(KeyModifiers::SUPER) {
            log::info!("🍎 CMD-TASTE ERKANNT! Modifier enthält SUPER flag");

            match key.code {
                KeyCode::Char(c) => {
                    log::info!("🍎 CMD+{} Event empfangen", c.to_uppercase());
                    match c {
                        'c' => log::info!("🍎 Das sollte COPY werden!"),
                        'v' => log::info!("🍎 Das sollte PASTE werden!"),
                        'x' => log::info!("🍎 Das sollte CUT werden!"),
                        'a' => log::info!("🍎 Das sollte SELECT ALL werden!"),
                        _ => log::info!("🍎 CMD+{} ist kein bekannter Shortcut", c),
                    }
                }
                other => log::info!("🍎 CMD+{:?} (non-char key)", other),
            }
        }
    }

    fn format_modifiers(&self, modifiers: KeyModifiers) -> String {
        let mut parts = Vec::new();

        if modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("SHIFT");
        }
        if modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("CTRL");
        }
        if modifiers.contains(KeyModifiers::ALT) {
            parts.push("ALT");
        }
        if modifiers.contains(KeyModifiers::SUPER) {
            parts.push("CMD");
        }

        if parts.is_empty() {
            "NONE".to_string()
        } else {
            parts.join("+")
        }
    }

    fn format_key_code(&self, code: KeyCode) -> String {
        match code {
            KeyCode::Char(c) => format!("'{}'", c),
            KeyCode::Enter => "ENTER".to_string(),
            KeyCode::Backspace => "BACKSPACE".to_string(),
            KeyCode::Delete => "DELETE".to_string(),
            KeyCode::Left => "LEFT".to_string(),
            KeyCode::Right => "RIGHT".to_string(),
            KeyCode::Up => "UP".to_string(),
            KeyCode::Down => "DOWN".to_string(),
            KeyCode::Home => "HOME".to_string(),
            KeyCode::End => "END".to_string(),
            KeyCode::PageUp => "PAGEUP".to_string(),
            KeyCode::PageDown => "PAGEDOWN".to_string(),
            KeyCode::Esc => "ESC".to_string(),
            other => format!("{:?}", other),
        }
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

        // Test control characters
        let ctrl_char = KeyEvent::new(KeyCode::Char('\x1B'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&ctrl_char), KeyAction::NoAction);

        // Test normal characters
        let normal_char = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&normal_char), KeyAction::InsertChar('a'));
    }

    #[test]
    fn test_cmd_shortcuts() {
        let mut manager = KeyboardManager::new();

        let cmd_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_c), KeyAction::CopySelection);

        let cmd_v = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_v), KeyAction::PasteBuffer);
    }
}
