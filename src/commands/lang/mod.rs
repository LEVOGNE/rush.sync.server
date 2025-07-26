// commands/lang/mod.rs - HAUPTEXPORT

pub mod command;
pub mod config;
pub mod manager;
pub mod persistence;

// ✅ CLEAN EXPORTS - nur was wirklich gebraucht wird
pub use command::LanguageCommand;
pub use manager::LanguageManager;
