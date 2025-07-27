// =====================================================
// FILE: commands/plugins.rs - OBJECT-SAFE PLUGIN SYSTEM
// =====================================================

use super::command::Command;
use super::registry::CommandRegistry;

/// ✅ PLUGIN SYSTEM für externe Commands - JETZT OBJECT-SAFE!
pub trait CommandPlugin {
    /// Lädt alle Commands des Plugins
    fn load_commands(&self) -> Vec<Box<dyn Command>>;

    /// Plugin Name
    fn name(&self) -> &'static str;

    /// Plugin Version (optional)
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Plugin ist verfügbar? (für conditional loading)
    fn is_available(&self) -> bool {
        true
    }
}

/// Plugin Manager verwaltet alle Plugins
pub struct PluginManager {
    plugins: Vec<Box<dyn CommandPlugin>>,
}

impl PluginManager {
    /// Neuer Plugin Manager
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Plugin laden
    pub fn load_plugin<T: CommandPlugin + 'static>(&mut self, plugin: T) -> &mut Self {
        if plugin.is_available() {
            log::debug!("Loading plugin: {} v{}", plugin.name(), plugin.version());
            self.plugins.push(Box::new(plugin));
        } else {
            log::warn!("Plugin not available: {}", plugin.name());
        }
        self
    }

    /// Alle Plugin-Commands zur Registry hinzufügen
    pub fn apply_to_registry(&mut self, registry: &mut CommandRegistry) {
        for plugin in &self.plugins {
            log::debug!("Applying plugin: {}", plugin.name());

            for command in plugin.load_commands() {
                registry.register_boxed(command);
            }
        }

        // Registry nach Plugin-Loading neu initialisieren
        registry.initialize();
    }

    /// Plugin-Statistiken
    pub fn stats(&self) -> (usize, usize) {
        let total = self.plugins.len();
        let available = self.plugins.iter().filter(|p| p.is_available()).count();
        (total, available)
    }

    /// Alle geladenen Plugins auflisten
    pub fn list_plugins(&self) -> Vec<(&str, &str, bool)> {
        self.plugins
            .iter()
            .map(|p| (p.name(), p.version(), p.is_available()))
            .collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================
// BEISPIEL-IMPLEMENTIERUNG - Network Plugin
// =====================================================

/*
use crate::commands::command::Command;
use crate::core::prelude::*;

// Beispiel: Ping Command
#[derive(Debug)]
pub struct PingCommand;

impl Command for PingCommand {
    fn name(&self) -> &'static str { "ping" }
    fn description(&self) -> &'static str { "Ping a host" }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("ping")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(host) => Ok(format!("Pinging {}...", host)),
            None => Ok("Usage: ping <host>".to_string()),
        }
    }

    fn priority(&self) -> u8 { 30 }
}

// Network Plugin
pub struct NetworkPlugin;

impl CommandPlugin for NetworkPlugin {
    fn name(&self) -> &'static str {
        "network"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn load_commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(PingCommand),
            // Box::new(WgetCommand),
            // Box::new(CurlCommand),
        ]
    }

    fn is_available(&self) -> bool {
        cfg!(feature = "network") // Nur mit network feature
    }
}

// NUTZUNG:
// let mut plugin_manager = PluginManager::new();
// plugin_manager.load_plugin(NetworkPlugin);
// plugin_manager.apply_to_registry(&mut registry);
*/
