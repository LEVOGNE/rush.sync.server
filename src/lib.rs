// src/lib.rs - CONFIG OWNERSHIP UPDATE
//! Rush Sync Terminal Application
//!
//! A modular terminal application with internationalization support
//! and extensible command system.

// Module definitions
pub mod commands;
pub mod core;
pub mod i18n;
pub mod input;
pub mod output;
pub mod setup;
pub mod ui;

// Essential re-exports (only the most commonly used types)
pub use core::config::Config;
pub use core::error::{AppError, Result};

/// Initializes and runs the terminal application
pub async fn run() -> Result<()> {
    let config = core::config::Config::load().await?;
    // ✅ REFERENZ wie ursprünglich
    let mut screen = ui::screen::ScreenManager::new(&config).await?;
    screen.run().await
}
