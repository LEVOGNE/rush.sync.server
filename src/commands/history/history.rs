// src/commands/history/history.rs
use crate::prelude::*;

pub struct HistoryCommand;

impl HistoryCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        command.trim().starts_with("history")
    }

    pub fn execute(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"-c" | &"--clear") => {
                Ok("__CLEAR_HISTORY__".to_string()) // Spezielles Signal zum Löschen
            }
            Some(&"-h" | &"--help") => Ok(
                "Verfügbare History Befehle:\n  history -c, --clear    Löscht die History"
                    .to_string(),
            ),
            _ => Ok("Unbekannter History Befehl. Nutze 'history -h' für Hilfe.".to_string()),
        }
    }
}
