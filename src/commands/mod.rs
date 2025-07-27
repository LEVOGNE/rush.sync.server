// src/commands/mod.rs - OHNE TRAITS

pub mod clear;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod restart;
pub mod version;

pub use handler::CommandHandler;
