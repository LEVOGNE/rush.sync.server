use super::command::Command;
use crate::core::prelude::*;
use std::collections::HashMap;

pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
    name_map: HashMap<String, usize>,
    available_cache: std::sync::RwLock<Vec<usize>>, // Neu: Cache für verfügbare Commands
    cache_dirty: std::sync::atomic::AtomicBool,     // Neu: Cache-Status
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            name_map: HashMap::new(),
            available_cache: std::sync::RwLock::new(Vec::new()),
            cache_dirty: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub fn register<T>(&mut self, command: T) -> &mut Self
    where
        T: Into<Box<dyn Command>>,
    {
        let boxed = command.into();
        let name = boxed.name().to_lowercase();
        let index = self.commands.len();

        self.commands.push(boxed);
        self.name_map.insert(name, index);

        // Cache invalidieren
        self.cache_dirty
            .store(true, std::sync::atomic::Ordering::Release);

        self
    }

    pub fn find_command(&self, input: &str) -> Option<&dyn Command> {
        let input = input.trim().to_lowercase();

        // Exakte Übereinstimmung (schnellster Pfad)
        if let Some(&index) = self.name_map.get(&input) {
            if let Some(cmd) = self.commands.get(index) {
                if cmd.is_available() {
                    return Some(cmd.as_ref());
                }
            }
        }

        // Cache-basierte Pattern-Matching
        self.update_available_cache_if_needed();

        if let Ok(cache) = self.available_cache.read() {
            for &index in cache.iter() {
                if let Some(cmd) = self.commands.get(index) {
                    if cmd.matches(&input) {
                        return Some(cmd.as_ref());
                    }
                }
            }
        }

        None
    }

    // Neue private Methode hinzufügen:
    fn update_available_cache_if_needed(&self) {
        if !self.cache_dirty.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }

        if let Ok(mut cache) = self.available_cache.write() {
            cache.clear();
            for (index, cmd) in self.commands.iter().enumerate() {
                if cmd.is_available() {
                    cache.push(index);
                }
            }
            self.cache_dirty
                .store(false, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn execute_sync(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        self.find_command(command).map(|cmd| cmd.execute_sync(args))
    }

    // src/commands/registry.rs
    pub async fn execute_async(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        match self.find_command(command) {
            Some(cmd) => Some(cmd.execute(args).await),
            None => None,
        }
    }

    // ✅ OPTIMIERT: Iterator-Chain statt collect
    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.update_available_cache_if_needed();

        if let Ok(cache) = self.available_cache.read() {
            cache
                .iter()
                .filter_map(|&index| {
                    self.commands
                        .get(index)
                        .map(|cmd| (cmd.name(), cmd.description()))
                })
                .collect()
        } else {
            // Fallback bei Lock-Fehler
            self.commands
                .iter()
                .filter(|cmd| cmd.is_available())
                .map(|cmd| (cmd.name(), cmd.description()))
                .collect()
        }
    }

    // ✅ VEREINFACHT: Weniger Felder zu debuggen
    pub fn debug_info(&self) -> String {
        format!(
            "CommandRegistry: {} commands registered",
            self.commands.len()
        )
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ✅ AUTO-IMPL: Ermöglicht register(MyCommand::new()) und register(Box::new(MyCommand::new()))
impl<T: Command> From<T> for Box<dyn Command> {
    fn from(cmd: T) -> Self {
        Box::new(cmd)
    }
}
