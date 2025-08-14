// =====================================================
// FILE: commands/mod.rs - MODULE CLEANUP
// =====================================================

pub mod clear;
pub mod command;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod log_level; // ✅ ADDED: Missing log_level module
pub mod performance; // ✅ ADDED: Missing performance module
pub mod plugins; // ✅ Plugins hinzugefügt
pub mod registry;
pub mod restart;
pub mod server;
pub mod theme; // ✅ ADDED: Missing theme module
pub mod version;

// ✅ CLEAN EXPORTS (macros entfernt da sie in lib.rs sind)
pub use command::Command;
pub use handler::CommandHandler;
pub use plugins::{CommandPlugin, PluginManager}; // ✅ Plugin exports
pub use registry::CommandRegistry;

// ✅ SERVER EXPORTS
pub use server::ServerCommand;
