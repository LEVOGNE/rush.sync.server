// =====================================================
// FILE: src/input/keyboard.rs - FIXED SHIFT SUPPORT
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
}

pub struct KeyboardManager {
    double_press_threshold: Duration,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            double_press_threshold: Duration::from_millis(DOUBLE_ESC_THRESHOLD),
        }
    }

    fn detect_broken_cmd_event(&self) -> bool {
        false // Placeholder implementation
    }

    pub fn get_action(&mut self, key: &KeyEvent) -> KeyAction {
        // Debug für Mac Cmd-Events
        if key.modifiers.contains(KeyModifiers::SUPER) {
            log::warn!(
                "🍎 RAW Cmd Event: code={:?}, modifiers={:?}, char={}",
                key.code,
                key.modifiers,
                match key.code {
                    KeyCode::Char(c) => format!("'{}'", c),
                    _ => "none".to_string(),
                }
            );
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

        // ✅ KORRIGIERTER MATCH - keine Duplikate mehr!
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

            // ========== 🚨 MAC CMD-EVENT FALLBACK (vor normalen Chars!) ==========
            (KeyCode::Char('v'), KeyModifiers::NONE) if self.detect_broken_cmd_event() => {
                log::warn!("🍎 Detected broken Cmd+V event, treating as paste");
                KeyAction::PasteBuffer
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) if self.detect_broken_cmd_event() => {
                log::warn!("🍎 Detected broken Cmd+C event, treating as copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) if self.detect_broken_cmd_event() => {
                log::warn!("🍎 Detected broken Cmd+A event, treating as move to start");
                KeyAction::MoveToStart
            }

            // ========== 🍎 MAC: CMD-SHORTCUTS ==========
            (KeyCode::Char('a'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+A → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Char('e'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+E → End");
                KeyAction::MoveToEnd
            }
            (KeyCode::Char('u'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+U → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Char('c'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+C → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+V → Paste");
                KeyAction::PasteBuffer
            }

            // ========== 🍎 MAC: ALT/OPTION-SHORTCUTS ==========
            (KeyCode::Char('a'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+A → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Char('e'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+E → End");
                KeyAction::MoveToEnd
            }
            (KeyCode::Char('u'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+U → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Char('c'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+C → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+V → Paste");
                KeyAction::PasteBuffer
            }

            // ========== 🖥️ WINDOWS/LINUX: CTRL-SHORTCUTS ==========
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+A → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+E → End");
                KeyAction::MoveToEnd
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+U → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+C → Copy");
                KeyAction::CopySelection
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+V → Paste");
                KeyAction::PasteBuffer
            }

            // ========== BACKSPACE-KOMBINATIONEN ==========
            (KeyCode::Backspace, KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+⌫ → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Backspace, KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+⌫ → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Backspace, KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+⌫ → Clear");
                KeyAction::ClearLine
            }

            // ========== BACKSLASH-ALTERNATIVEN ==========
            (KeyCode::Char('\\'), KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+\\ → Clear");
                KeyAction::ClearLine
            }
            (KeyCode::Char('\\'), KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+\\ → Clear");
                KeyAction::ClearLine
            }

            // ========== PFEILTASTEN MIT MODIFIERS ==========
            (KeyCode::Left, KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+← → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Right, KeyModifiers::SUPER) => {
                log::debug!("🍎 Cmd+→ → End");
                KeyAction::MoveToEnd
            }
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+← → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Right, KeyModifiers::CONTROL) => {
                log::debug!("🖥️ Ctrl+→ → End");
                KeyAction::MoveToEnd
            }
            (KeyCode::Left, KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+← → Start");
                KeyAction::MoveToStart
            }
            (KeyCode::Right, KeyModifiers::ALT) => {
                log::debug!("🍎 Opt+→ → End");
                KeyAction::MoveToEnd
            }

            // ========== NORMALE ZEICHEN-EINGABE (MUSS AM ENDE STEHEN!) ==========
            (KeyCode::Char(c), KeyModifiers::NONE) => KeyAction::InsertChar(c),
            (KeyCode::Char(c), KeyModifiers::SHIFT) => KeyAction::InsertChar(c),

            // ========== FALLBACK für unbekannte Kombinationen ==========
            (code, modifiers) => {
                log::debug!("❓ Unhandled key combination: {:?} + {:?}", code, modifiers);
                KeyAction::NoAction
            }
        }
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}

// ✅ TESTS um sicherzustellen, dass Shift funktioniert
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_shift_support() {
        let mut manager = KeyboardManager::new();

        // Lowercase ohne Shift
        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&key_a), KeyAction::InsertChar('a'));

        // Uppercase mit Shift (das sollte jetzt funktionieren!)
        let key_a_upper = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);
        assert_eq!(manager.get_action(&key_a_upper), KeyAction::InsertChar('A'));

        // Zahlen mit Shift (für Symbole)
        let key_exclamation = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT);
        assert_eq!(
            manager.get_action(&key_exclamation),
            KeyAction::InsertChar('!')
        );

        // Symbols mit Shift
        let key_at = KeyEvent::new(KeyCode::Char('@'), KeyModifiers::SHIFT);
        assert_eq!(manager.get_action(&key_at), KeyAction::InsertChar('@'));
    }

    #[test]
    fn test_mac_specific_shortcuts() {
        let mut manager = KeyboardManager::new();

        // ✅ MAC: Option-basierte Shortcuts (funktionieren besser als Cmd)
        let opt_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_a), KeyAction::MoveToStart);

        let opt_e = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_e), KeyAction::MoveToEnd);

        let opt_u = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_u), KeyAction::ClearLine);

        // ✅ MAC: Option+Backspace und Cmd+Backspace für Zeile löschen
        let opt_backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_backspace), KeyAction::ClearLine);

        let cmd_backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_backspace), KeyAction::ClearLine);

        let ctrl_backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_backspace), KeyAction::ClearLine);

        // ✅ BONUS: Backslash-Alternativen (falls Backspace nicht funktioniert)
        let cmd_backslash = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_backslash), KeyAction::ClearLine);

        let opt_backslash = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_backslash), KeyAction::ClearLine);

        // ✅ WINDOWS/LINUX: Ctrl-basierte Shortcuts
        let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_a), KeyAction::MoveToStart);

        let ctrl_e = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_e), KeyAction::MoveToEnd);

        // ✅ PFEILTASTEN: Verschiedene Modifier
        let cmd_left = KeyEvent::new(KeyCode::Left, KeyModifiers::SUPER);
        assert_eq!(manager.get_action(&cmd_left), KeyAction::MoveToStart);

        let opt_left = KeyEvent::new(KeyCode::Left, KeyModifiers::ALT);
        assert_eq!(manager.get_action(&opt_left), KeyAction::MoveToStart);

        let ctrl_left = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_left), KeyAction::MoveToStart);
    }

    #[test]
    fn test_special_characters() {
        let mut manager = KeyboardManager::new();

        // Deutsche Umlaute
        let key_ä = KeyEvent::new(KeyCode::Char('ä'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&key_ä), KeyAction::InsertChar('ä'));

        let key_ae_upper = KeyEvent::new(KeyCode::Char('Ä'), KeyModifiers::SHIFT);
        assert_eq!(
            manager.get_action(&key_ae_upper),
            KeyAction::InsertChar('Ä')
        );

        // Emoji und Unicode
        let key_emoji = KeyEvent::new(KeyCode::Char('🚀'), KeyModifiers::NONE);
        assert_eq!(manager.get_action(&key_emoji), KeyAction::InsertChar('🚀'));
    }
}
