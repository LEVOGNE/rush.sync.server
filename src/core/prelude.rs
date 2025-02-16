// src/core/prelude.rs

// Standard Library Imports
pub use std::{
    fs,
    io::{self, Stdout, Write},
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};

// Crossterm Imports
pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::ResetColor,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

// Ratatui Imports
pub use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Terminal,
};

// External Library Imports
pub use lazy_static::lazy_static;
pub use log::{Level, LevelFilter, Metadata, Record};
pub use serde::{Deserialize, Serialize};
pub use unicode_segmentation::UnicodeSegmentation;

// Internal Imports - Core
pub use crate::core::{
    config::Config,
    error::{AppError, Result},
};

// Internal Imports - Input
pub use crate::input::{
    event::{AppEvent, EventHandler},
    input::InputState,
};

// Internal Imports - Output
pub use crate::output::{message::MessageManager, output::create_output_widget};

// Internal Imports - UI
pub use crate::ui::{color::AppColor, terminal::TerminalManager, widget::Widget};

// Internal Imports - i18n
pub use crate::i18n::{get_translation, get_translation_details, TranslationError};

// Type Aliases
pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;
