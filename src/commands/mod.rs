// =====================================================
// FILE: commands/mod.rs - CLEAN VERSION OHNE UNNÖTIGE PLUGINS
// =====================================================

pub mod clear;
pub mod command;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod log_level;
pub mod registry;
pub mod restart;
pub mod theme;
pub mod version;

pub mod test;

// ✅ CLEAN EXPORTS - Nur was wirklich gebraucht wird
pub use command::Command;
pub use handler::CommandHandler;
pub use registry::CommandRegistry;
