// src/core/prelude.rs - KORRIGIERT

// Core essentials - ÜBERALL gebraucht
pub use crate::core::config::Config;
pub use crate::core::error::{AppError, Result};

// Standard library essentials
pub use std::io::{self, Write};
pub use std::time::{Duration, Instant};

// Crossterm basics (nur die wichtigsten)
pub use crossterm::event::{KeyCode, KeyEvent};

// Ratatui basics
pub use ratatui::style::Color;

// ✅ i18n - NUR DIE ESSENTIALS
pub use crate::i18n::{get_translation, TranslationError};

// ✅ Macros werden automatisch verfügbar durch #[macro_export]
