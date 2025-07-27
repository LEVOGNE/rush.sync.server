// src/commands/clear/clear.rs
use crate::core::prelude::*;

#[derive(Debug)]
pub struct ClearCommand;

impl ClearCommand {
    pub fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "clear" | "cls")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok("__CLEAR__".to_string())
    }

    crate::async_fallback!();
}
