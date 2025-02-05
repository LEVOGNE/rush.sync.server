// src/commands/mod.rs
pub mod exit;
pub mod handler;
pub mod lang;
pub use handler::CommandHandler; // re-export des Handlers
