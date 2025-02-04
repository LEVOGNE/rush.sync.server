use crate::prelude::*;

pub struct ExitCommand;

impl ExitCommand {
    // new() ist hier optional
    pub fn new() -> Self {
        Self
    }

    /// Führt den Exit-Befehl aus, liefert nur ein kurzes Signal als Nachricht
    pub fn execute(&self) -> Result<String> {
        Ok(String::new()) // oder z. B. Ok("exit".to_string())
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "exit" | "quit")
    }
}
