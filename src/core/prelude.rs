// src/core/prelude.rs - VEREINFACHT (mit i18n)

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

// i18n - da mehrsprachig gewünscht
pub use crate::i18n::{get_translation, get_translation_details, set_language, TranslationError};

// Das war's! Alles andere wird spezifisch importiert wo es gebraucht wird.
