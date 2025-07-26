// =====================================================
// FILE: commands/history/mod.rs - KORRIGIERT
// =====================================================

pub mod config;
pub mod events;
pub mod history;
pub mod keyboard;
pub mod manager;

// Clean Re-exports
pub use config::HistoryConfig;
pub use events::{HistoryEvent, HistoryEventHandler};
pub use history::HistoryCommand;
pub use keyboard::{HistoryAction, HistoryKeyboardHandler};
pub use manager::HistoryManager;
