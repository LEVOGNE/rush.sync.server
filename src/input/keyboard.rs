use crate::core::constants::DOUBLE_ESC_THRESHOLD;
use crate::core::prelude::*;
use crossterm::event::KeyModifiers;

// Zentrale Enum für alle Tastaturaktionen
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    // Navigation
    MoveLeft,
    MoveRight,
    MoveToStart,
    MoveToEnd,

    // Edition
    InsertChar(char),
    Backspace,
    Delete,

    // History
    HistoryPrevious,
    HistoryNext,

    // Control
    Submit,
    Cancel,
    Quit,

    // Spezielle Aktionen
    ClearLine,
    CopySelection,
    PasteBuffer,

    // Keine Aktion
    NoAction,

    // Scrollen
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
}

// Konfigurierbare Hotkey-Struktur
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: KeyEvent,
    pub action: KeyAction,
    pub description: String,
}

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref LAST_ESC_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
}

pub struct KeyboardManager {
    bindings: Vec<KeyBinding>,
    double_press_threshold: Duration,
}

impl KeyboardManager {
    pub fn new() -> Self {
        let mut km = Self {
            bindings: Vec::new(),
            double_press_threshold: Duration::from_millis(DOUBLE_ESC_THRESHOLD),
        };
        km.setup_default_bindings();
        km
    }

    fn setup_default_bindings(&mut self) {
        // Standard-Tastenbelegungen
        self.add_binding(
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyAction::MoveLeft,
            "Cursor nach links bewegen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            KeyAction::MoveRight,
            "Cursor nach rechts bewegen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
            KeyAction::MoveToStart,
            "Zum Zeilenanfang springen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
            KeyAction::MoveToEnd,
            "Zum Zeilenende springen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyAction::Submit,
            "Eingabe bestätigen",
        );

        // Alt + Pfeiltasten für Scrollen
        self.add_binding(
            KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT),
            KeyAction::ScrollUp,
            "Eine Zeile nach oben scrollen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT),
            KeyAction::ScrollDown,
            "Eine Zeile nach unten scrollen",
        );

        // PageUp/PageDown für seitenweises Scrollen
        self.add_binding(
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            KeyAction::PageUp,
            "Eine Seite nach oben scrollen",
        );
        self.add_binding(
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            KeyAction::PageDown,
            "Eine Seite nach unten scrollen",
        );
    }

    pub fn add_binding(&mut self, key: KeyEvent, action: KeyAction, description: &str) {
        self.bindings.push(KeyBinding {
            key,
            action,
            description: description.to_string(),
        });
    }

    pub fn get_action(&mut self, key: &KeyEvent) -> KeyAction {
        // Spezielle Behandlung für ESC
        if key.code == KeyCode::Esc {
            let now = Instant::now();
            let mut last_press: std::sync::MutexGuard<Option<Instant>> =
                LAST_ESC_PRESS.lock().unwrap();

            if let Some(prev_press) = *last_press {
                if now.duration_since(prev_press) <= self.double_press_threshold {
                    *last_press = None; // Reset Timer
                    return KeyAction::Quit;
                }
            }

            // Erster ESC-Druck
            *last_press = Some(now);
            return KeyAction::NoAction;
        }

        // Normales Binding-Checking für andere Tasten
        for binding in &self.bindings {
            if binding.key.code == key.code && binding.key.modifiers == key.modifiers {
                return binding.action.clone();
            }
        }

        // Standardverarbeitung für nicht gebundene Tasten
        match key.code {
            KeyCode::Char(c) => KeyAction::InsertChar(c),
            KeyCode::Backspace => KeyAction::Backspace,
            KeyCode::Delete => KeyAction::Delete,
            KeyCode::Up => KeyAction::HistoryPrevious,
            KeyCode::Down => KeyAction::HistoryNext,
            _ => KeyAction::NoAction,
        }
    }

    // Hilfsmethode zum Abrufen aller verfügbaren Tastenkombinationen
    pub fn get_all_bindings(&self) -> Vec<&KeyBinding> {
        self.bindings.iter().collect()
    }
}

// Implementierung für benutzerdefinierte Tastenkombinationen
impl KeyBinding {
    pub fn new(key: KeyEvent, action: KeyAction, description: &str) -> Self {
        Self {
            key,
            action,
            description: description.to_string(),
        }
    }

    pub fn matches(&self, key: &KeyEvent) -> bool {
        self.key.code == key.code && self.key.modifiers == key.modifiers
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}
