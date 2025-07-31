use super::command::Command;
use crate::core::prelude::*;
use std::collections::HashMap;

pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
    name_map: HashMap<String, usize>,
    initialized: bool,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            name_map: HashMap::new(),
            initialized: false,
        }
    }

    pub fn register<T: Command>(&mut self, command: T) -> &mut Self {
        let name = command.name().to_lowercase();
        let index = self.commands.len();

        self.commands.push(Box::new(command));
        self.name_map.insert(name, index);
        self
    }

    pub fn register_boxed(&mut self, command: Box<dyn Command>) -> &mut Self {
        let name = command.name().to_lowercase();
        let index = self.commands.len();

        self.commands.push(command);
        self.name_map.insert(name, index);
        self
    }

    pub fn initialize(&mut self) -> &mut Self {
        if self.initialized {
            return self;
        }

        self.name_map.clear();
        for (new_idx, cmd) in self.commands.iter().enumerate() {
            self.name_map.insert(cmd.name().to_lowercase(), new_idx);
        }

        self.initialized = true;
        self
    }

    pub fn find_command(&self, input: &str) -> Option<&dyn Command> {
        let input = input.trim().to_lowercase();

        // Exakte Übereinstimmung
        if let Some(&index) = self.name_map.get(&input) {
            return self.commands.get(index).map(|cmd| cmd.as_ref());
        }

        // Pattern matching
        for cmd in &self.commands {
            if cmd.is_available() && cmd.matches(&input) {
                return Some(cmd.as_ref());
            }
        }

        None
    }

    // FIX: Correct Result type with Error parameter
    pub fn execute_sync(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        self.find_command(command).map(|cmd| cmd.execute_sync(args))
    }

    // FIX: Correct Result type with Error parameter
    pub async fn execute_async(&self, command: &str, args: &[&str]) -> Option<Result<String>> {
        if let Some(cmd) = self.find_command(command) {
            Some(cmd.execute_async(args).await)
        } else {
            None
        }
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.commands
            .iter()
            .filter(|cmd| cmd.is_available())
            .map(|cmd| (cmd.name(), cmd.description()))
            .collect()
    }

    pub fn debug_info(&self) -> String {
        format!(
            "CommandRegistry: {} commands, initialized: {}",
            self.commands.len(),
            self.initialized
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
