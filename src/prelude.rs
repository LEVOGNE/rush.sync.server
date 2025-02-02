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
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

// External Library Imports
pub use lazy_static::lazy_static;
pub use log::{Level, LevelFilter, Metadata, Record};
pub use serde::{Deserialize, Serialize};
pub use unicode_segmentation::UnicodeSegmentation;

// Internal Imports - Core Types
pub use crate::{
    color::AppColor,
    config::Config,
    error::{AppError, Result},
    event::{AppEvent, EventHandler}, // Geändert von Event, Events
    terminal::TerminalManager,
};

// Internal Imports - Features
pub use crate::{
    input::InputState, message::MessageManager, output::create_output_widget, widget::Widget,
};

// Type Aliases
pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;
