// src/commands/mod.rs
pub mod clear;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod version;

pub use handler::CommandHandler;
