pub mod config;
pub mod handlers;
pub mod manager;
pub mod shared;
pub mod types;
pub mod utils;

// Re-exports für einfache Verwendung
pub use manager::ServerManager;
pub use types::{ServerInfo, ServerStatus};
