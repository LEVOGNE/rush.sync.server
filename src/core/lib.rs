// src/core/lib.rs
pub mod core {
    pub mod config;
    pub mod constants;
    pub mod error;
    pub mod prelude;
}

pub mod ui {
    pub mod color;
    pub mod cursor;
    pub mod screen;
    pub mod terminal;
    pub mod widget;
}

pub mod input {
    pub mod event;
    pub mod input;
    pub mod keyboard;
}

pub mod output {
    pub mod logging;
    pub mod message;
    pub mod output;
    pub mod scroll;
}

pub mod setup {
    pub mod setup_toml;
}

// Re-exports
pub use crate::core::prelude::*;
