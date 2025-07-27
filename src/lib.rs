// src/lib.rs - KORRIGIERT
//! Rush Sync Terminal Application
//!
//! A modular terminal application with internationalization support
//! and extensible command system.

// ✅ Macros ZUERST definieren - vor allen Modulen
#[macro_export]
macro_rules! async_fallback {
    () => {
        pub async fn execute_async(&self, args: &[&str]) -> crate::core::error::Result<String> {
            self.execute_sync(args)
        }

        pub fn supports_async(&self) -> bool {
            false
        }
    };
}

#[macro_export]
macro_rules! impl_default {
    ($type:ty, $body:expr) => {
        impl Default for $type {
            fn default() -> Self {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! matches_exact {
    ($cmd:expr, $($pattern:literal)|+) => {
        matches!($cmd.trim().to_lowercase().as_str(), $($pattern)|+)
    };
}

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
    let mut screen = ui::screen::ScreenManager::new(&config).await?;
    screen.run().await
}
