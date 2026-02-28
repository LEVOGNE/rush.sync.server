use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug, Default)]
pub struct HelpCommand;

impl HelpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Show all available commands"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd == "help" || cmd == "?" || cmd == "commands" || cmd == "list-commands"
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let handler = crate::commands::CommandHandler::new();

        match args.first() {
            Some(&"--simple" | &"-s") => Ok(self.create_simple_list(&handler)),
            Some(&"--detailed" | &"-d") => Ok(self.create_detailed_list(&handler)),
            None => Ok(self.create_formatted_list(&handler)),
            Some(&command_name) => Ok(self.show_command_help(command_name, &handler)),
        }
    }

    fn priority(&self) -> u8 {
        95
    }
}

impl HelpCommand {
    /// Look up the localized description for a command, falling back to the original
    fn get_localized_description(&self, command_name: &str, original_description: &str) -> String {
        // Normalize command name for i18n key lookup
        let normalized_name = command_name.replace("-", "_");
        let description_key = format!("system.commands.{}.description", normalized_name);

        if crate::i18n::has_translation(&description_key) {
            get_command_translation(&description_key, &[])
        } else {
            log::debug!(
                "No translation found for key '{}', using original description",
                description_key
            );
            original_description.to_string()
        }
    }

    /// Build the formatted default help list
    fn create_formatted_list(&self, handler: &crate::commands::CommandHandler) -> String {
        let commands = handler.list_commands();

        if commands.is_empty() {
            return get_command_translation("system.commands.help.no_commands", &[]);
        }

        let mut result = String::new();
        result.push_str(&get_command_translation("system.commands.help.header", &[]));
        result.push_str("\n\n");

        let mut categorized = std::collections::BTreeMap::new();

        for (name, original_description) in commands {
            let category_key = self.determine_category(name);
            let localized_description = self.get_localized_description(name, original_description);

            categorized
                .entry(category_key)
                .or_insert_with(Vec::new)
                .push((name, localized_description));
        }

        for (category_key, commands) in categorized {
            let category_translation_key =
                format!("system.commands.help.category.{}", category_key);

            let category_name = if crate::i18n::has_translation(&category_translation_key) {
                get_command_translation(&category_translation_key, &[])
            } else {
                log::debug!(
                    "No translation for category '{}', using fallback",
                    category_key
                );
                self.get_fallback_category_name(category_key)
            };

            result.push_str(&format!("{}:\n", category_name));

            for (name, description) in commands {
                result.push_str(&format!("  {:12} - {}\n", name, description));
            }
            result.push('\n');
        }

        result.push_str(&get_command_translation("system.commands.help.footer", &[]));
        result
    }

    /// Determine the category for a command by name prefix
    fn determine_category(&self, command_name: &str) -> &'static str {
        match command_name {
            name if name.starts_with("start")
                || name.starts_with("stop")
                || name.starts_with("restart") =>
            {
                "server_control"
            }
            name if name.starts_with("create") || name.starts_with("list") => "server_management",
            name if name.starts_with("remote") || name.starts_with("sync") => "deployment",
            name if name.starts_with("cleanup") || name.starts_with("recover") => "maintenance",
            name if name.starts_with("theme")
                || name.starts_with("lang")
                || name.starts_with("log-level") =>
            {
                "configuration"
            }
            name if name.starts_with("help")
                || name.starts_with("version")
                || name.starts_with("history") =>
            {
                "information"
            }
            name if name.starts_with("exit") || name.starts_with("clear") => "system",
            _ => "other",
        }
    }

    /// Fallback category names when i18n key is missing
    fn get_fallback_category_name(&self, category_key: &str) -> String {
        match category_key {
            "server_control" => "Server Control".to_string(),
            "server_management" => "Server Management".to_string(),
            "deployment" => "Deployment & Sync".to_string(),
            "maintenance" => "Maintenance".to_string(),
            "configuration" => "Configuration".to_string(),
            "information" => "Information".to_string(),
            "system" => "System".to_string(),
            "other" => "Other".to_string(),
            _ => category_key.to_string(),
        }
    }

    /// Build a comma-separated simple command list
    fn create_simple_list(&self, handler: &crate::commands::CommandHandler) -> String {
        let commands = handler.list_commands();
        let names: Vec<&str> = commands.iter().map(|(name, _)| *name).collect();
        let names_str = names.join(", ");

        get_command_translation("system.commands.help.simple_list", &[&names_str])
    }

    /// Build a detailed command list with labels and separators
    fn create_detailed_list(&self, handler: &crate::commands::CommandHandler) -> String {
        let commands = handler.list_commands();
        let mut result = String::new();

        result.push_str(&get_command_translation(
            "system.commands.help.detailed_header",
            &[],
        ));
        result.push('\n');
        result.push_str(&get_command_translation(
            "system.commands.help.detailed_separator",
            &[],
        ));
        result.push_str("\n\n");

        for (name, original_description) in commands {
            let localized_description = self.get_localized_description(name, original_description);

            let command_label = get_command_translation("system.commands.help.command_label", &[]);
            let description_label =
                get_command_translation("system.commands.help.description_label", &[]);
            let usage_label = get_command_translation("system.commands.help.usage_label", &[]);
            let separator = get_command_translation("system.commands.help.command_separator", &[]);

            result.push_str(&format!("{} {}\n", command_label, name.to_uppercase()));
            result.push_str(&format!(
                "{} {}\n",
                description_label, localized_description
            ));
            result.push_str(&format!("{} {} [options]\n", usage_label, name));
            result.push_str(&format!("{}\n", separator));
        }

        result
    }

    /// Show help for a specific command
    fn show_command_help(
        &self,
        command_name: &str,
        handler: &crate::commands::CommandHandler,
    ) -> String {
        let commands = handler.list_commands();

        for (name, original_description) in commands {
            if name.eq_ignore_ascii_case(command_name) {
                let localized_description =
                    self.get_localized_description(name, original_description);

                return get_command_translation(
                    "system.commands.help.specific_help_template",
                    &[name, &localized_description, name, name],
                );
            }
        }

        get_command_translation("system.commands.help.command_not_found", &[command_name])
    }
}
