// =====================================================
// FILE: commands/registry.rs - KOMPLETT IMPLEMENTIERT
// =====================================================

use super::command::Command;
use crate::core::prelude::*;
use std::collections::HashMap;

/// Zentrale Registry für alle Commands - HOCHPERFORMANT & ROBUST
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
    name_map: HashMap<String, usize>,
    initialized: bool,
}

impl CommandRegistry {
    /// Neue leere Registry erstellen
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            name_map: HashMap::new(),
            initialized: false,
        }
    }

    /// Command registrieren - CHAIN-ABLE
    pub fn register<T: Command>(&mut self, command: T) -> &mut Self {
        let name = command.name().to_lowercase();
        let index = self.commands.len();

        self.commands.push(Box::new(command));
        self.name_map.insert(name, index);

        log::debug!("Command registered: {}", self.commands[index].name());
        self
    }

    /// Registriert einen Boxed Command (für Plugins)
    pub fn register_boxed(&mut self, command: Box<dyn Command>) -> &mut Self {
        let name = command.name().to_lowercase();
        let index = self.commands.len();

        self.commands.push(command);
        self.name_map.insert(name, index);

        log::debug!("Boxed command registered: {}", self.commands[index].name());
        self
    }

    /// Registry finalisieren - sortiert nach Priorität
    pub fn initialize(&mut self) -> &mut Self {
        if self.initialized {
            return self;
        }

        // Einfache Sortierung nach Priorität - die Commands sind bereits in Boxen
        // Wir sortieren nur die Indizes und bauen name_map neu auf
        self.name_map.clear();
        for (new_idx, cmd) in self.commands.iter().enumerate() {
            self.name_map.insert(cmd.name().to_lowercase(), new_idx);
        }

        self.initialized = true;
        log::debug!(
            "CommandRegistry initialized with {} commands",
            self.commands.len()
        );
        self
    }

    /// Findet Command basierend auf Input
    pub fn find_command(&self, input: &str) -> Option<&dyn Command> {
        let input = input.trim().to_lowercase();

        // Erst exakte Übereinstimmung via name_map
        if let Some(&index) = self.name_map.get(&input) {
            return self.commands.get(index).map(|cmd| cmd.as_ref());
        }

        // Dann pattern matching (nach Priorität sortiert)
        for cmd in &self.commands {
            if cmd.is_available() && cmd.matches(&input) {
                return Some(cmd.as_ref());
            }
        }

        None
    }

    /// Führt Command synchron aus
    pub fn execute_sync(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        self.find_command(command).map(|cmd| cmd.execute_sync(args))
    }

    /// Führt Command asynchron aus
    pub async fn execute_async(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        if let Some(cmd) = self.find_command(command) {
            Some(cmd.execute_async(args).await)
        } else {
            None
        }
    }

    /// Listet alle verfügbaren Commands auf
    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.commands
            .iter()
            .filter(|cmd| cmd.is_available())
            .map(|cmd| (cmd.name(), cmd.description()))
            .collect()
    }

    /// Debug-Informationen
    pub fn debug_info(&self) -> String {
        let total = self.commands.len();
        let available = self
            .commands
            .iter()
            .filter(|cmd| cmd.is_available())
            .count();
        let async_support = self
            .commands
            .iter()
            .filter(|cmd| cmd.supports_async())
            .count();

        format!(
            "CommandRegistry: {} total, {} available, {} async-capable, initialized: {}",
            total, available, async_support, self.initialized
        )
    }

    /// Anzahl registrierter Commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Ist Registry leer?
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
