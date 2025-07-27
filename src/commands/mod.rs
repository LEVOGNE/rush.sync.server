// =====================================================
// FILE: commands/mod.rs - MODULE CLEANUP
// =====================================================

pub mod clear;
pub mod command;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod log_level;
pub mod performance;
pub mod plugins;
pub mod registry;
pub mod restart;
pub mod version;

// ✅ CLEAN EXPORTS (macros entfernt da sie in lib.rs sind)
pub use command::Command;
pub use handler::CommandHandler;
pub use plugins::{CommandPlugin, PluginManager};
pub use registry::CommandRegistry;

// =====================================================
// ERWEITERTE PLUGIN EXAMPLE - WIE ES GENUTZT WIRD
// =====================================================

/*
// ✅ BEISPIEL: Neues Network Plugin

// FILE: commands/network/mod.rs
pub mod ping;
pub mod wget;

pub use ping::PingCommand;
pub use wget::WgetCommand;

// Network Plugin Implementation
use crate::commands::{Command, CommandPlugin};

pub struct NetworkPlugin;

impl CommandPlugin for NetworkPlugin {
    fn name(&self) -> &'static str {
        "network"
    }

    fn load_commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(PingCommand),
            Box::new(WgetCommand),
        ]
    }
}

// ✅ NUTZUNG in main.rs oder wo auch immer:

use crate::commands::{CommandHandler, PluginManager};
use crate::commands::network::NetworkPlugin;

let mut handler = CommandHandler::new();

// Option 1: Plugin zur Laufzeit hinzufügen
let mut plugin_manager = PluginManager::new();
plugin_manager.load_plugin(NetworkPlugin);
plugin_manager.apply_to_registry(&mut handler.registry);

// Option 2: Mit erweiterten Macros (optional)
let handler = CommandHandler::with_registry(
    create_full_registry_with_plugins!(NetworkPlugin)
);

*/

// =====================================================
// PERFORMANCE TIPP - COMMAND CACHING
// =====================================================

/*
// ✅ OPTIONAL: Command Caching für bessere Performance

use std::collections::HashMap;

impl CommandRegistry {
    cache: HashMap<String, usize>, // Cache command name -> index

    pub fn find_command_cached(&self, input: &str) -> Option<&dyn Command> {
        // Cache lookup first, dann normale find_command logic
        if let Some(&index) = self.cache.get(input) {
            return self.commands.get(index).map(|cmd| cmd.as_ref());
        }

        // Normal lookup + cache update
        if let Some(cmd) = self.find_command(input) {
            // Update cache
            self.cache.insert(input.to_string(), /* index */);
            Some(cmd)
        } else {
            None
        }
    }
}
*/

// =====================================================
// DEBUG COMMAND EXAMPLE - NUR IN DEBUG BUILDS
// =====================================================

/*
// ✅ BEISPIEL: Debug Command nur in Development

#[derive(Debug)]
pub struct DebugCommand;

impl Command for DebugCommand {
    fn name(&self) -> &'static str { "debug" }
    fn description(&self) -> &'static str { "Debug utilities" }

    fn matches(&self, command: &str) -> bool {
        command == "debug"
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"registry") => {
                // Registry debug info von handler
                Ok("Registry debug info...".to_string())
            }
            Some(&"commands") => {
                // Liste alle verfügbaren commands
                Ok("Available commands: ...".to_string())
            }
            _ => Ok("Debug commands: registry, commands".to_string())
        }
    }

    fn is_available(&self) -> bool {
        cfg!(debug_assertions) // ✅ Nur in Debug builds
    }

    fn priority(&self) -> u8 { 10 } // Niedrigste Priorität
}

// Dann in create_full_registry! hinzufügen:
#[cfg(debug_assertions)]
register_commands!(registry, DebugCommand);
*/
