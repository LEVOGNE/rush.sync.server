// ✅ CORE ESSENTIALS - überall gebraucht
pub use crate::core::config::Config;
pub use crate::core::error::{AppError, Result};

// ✅ STANDARD LIBRARY ESSENTIALS
pub use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::time::{Duration, Instant};

// ✅ CROSSTERM BASICS (nur die wichtigsten)
pub use crossterm::event::{KeyCode, KeyEvent};

// ✅ RATATUI BASICS
pub use ratatui::style::Color;

// ✅ i18n INTEGRATION - VOLLSTÄNDIG
pub use crate::i18n::{
    clear_translation_cache, get_available_languages, get_color_category_for_display,
    get_command_translation, get_current_language, get_translation, has_translation, set_language,
    TranslationError,
};

// ✅ i18n MAKROS - für einfache Nutzung
pub use crate::{t, tc};

// ✅ ZUSÄTZLICHE UTILITIES für häufige Operationen
pub use crate::ui::color::AppColor;

// ✅ LOGGING INTEGRATION - damit log! Makros i18n nutzen können
pub use log::{debug, error, info, trace, warn};

// ✅ RE-EXPORTS für bessere API
pub use crate::commands::{Command, CommandHandler, CommandRegistry};
pub use crate::input::keyboard::KeyAction;
pub use crate::output::display::MessageDisplay;
