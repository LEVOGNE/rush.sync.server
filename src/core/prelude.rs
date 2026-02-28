// Core essentials
pub use crate::core::config::Config;
pub use crate::core::error::{AppError, Result};
pub use crate::core::helpers::{get_config, read_lock, write_lock};

// Standard library essentials
pub use std::collections::HashMap;
pub use std::io::{self, Write};
pub use std::time::{Duration, Instant};

// Crossterm basics
pub use crossterm::event::{KeyCode, KeyEvent};

// Ratatui basics
pub use ratatui::style::Color;

// i18n integration
pub use crate::i18n::{
    clear_translation_cache, get_available_languages, get_color_category_for_display,
    get_command_translation, get_current_language, get_translation, has_translation, set_language,
    TranslationError,
};

// i18n macros
pub use crate::{t, tc};

// Additional utilities
pub use crate::ui::color::AppColor;

// Logging integration
pub use log::{debug, error, info, trace, warn};

// Re-exports
pub use crate::commands::{Command, CommandHandler, CommandRegistry};
pub use crate::input::keyboard::KeyAction;
pub use crate::output::display::MessageDisplay;
