// ## FILE: input/keyboard.rs - NUR KeyBinding struct entfernt
// ## BEGIN ##
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
    HistoryPrevious,
    HistoryNext,
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

    pub fn get_action(&mut self, key: &KeyEvent) -> KeyAction {
        // Spezielle Behandlung für ESC
        if key.code == KeyCode::Esc {
            let now = Instant::now();
            let mut last_press: std::sync::MutexGuard<Option<Instant>> =
                LAST_ESC_PRESS.lock().unwrap();

            if let Some(prev_press) = *last_press {
                if now.duration_since(prev_press) <= self.double_press_threshold {
                    *last_press = None;
                    return KeyAction::Quit;
                }
            }

            *last_press = Some(now);
            return KeyAction::NoAction;
        }

        // ✅ DIREKTE Mappings - kein KeyBinding overhead
        match (key.code, key.modifiers) {
            (KeyCode::Left, KeyModifiers::NONE) => KeyAction::MoveLeft,
            (KeyCode::Right, KeyModifiers::NONE) => KeyAction::MoveRight,
            (KeyCode::Home, KeyModifiers::NONE) => KeyAction::MoveToStart,
            (KeyCode::End, KeyModifiers::NONE) => KeyAction::MoveToEnd,
            (KeyCode::Enter, KeyModifiers::NONE) => KeyAction::Submit,
            (KeyCode::Up, KeyModifiers::SHIFT) => KeyAction::ScrollUp,
            (KeyCode::Down, KeyModifiers::SHIFT) => KeyAction::ScrollDown,
            (KeyCode::PageUp, KeyModifiers::NONE) => KeyAction::PageUp,
            (KeyCode::PageDown, KeyModifiers::NONE) => KeyAction::PageDown,
            (KeyCode::Char(c), KeyModifiers::NONE) => KeyAction::InsertChar(c),
            (KeyCode::Backspace, KeyModifiers::NONE) => KeyAction::Backspace,
            (KeyCode::Delete, KeyModifiers::NONE) => KeyAction::Delete,
            (KeyCode::Up, KeyModifiers::NONE) => KeyAction::HistoryPrevious,
            (KeyCode::Down, KeyModifiers::NONE) => KeyAction::HistoryNext,
            _ => KeyAction::NoAction,
        }
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}
// ## END ##
