use crate::prelude::*;

pub struct ExitCommand;

impl ExitCommand {
    pub fn new() -> Self {
        Self
    }

    /// Führt den Exit-Befehl aus, gibt aber erst eine Rückfrage zurück
    pub fn execute(&self) -> Result<String> {
        // Rückgabe einer Bestätigungsanfrage, die dann im Input-State verarbeitet wird
        Ok("__CONFIRM_EXIT__Möchten Sie das Programm wirklich beenden? (j/n)".to_string())
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "exit" | "q")
    }
}
