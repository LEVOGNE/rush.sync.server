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
    /// Get detailed usage info for a command
    fn get_command_usage(command_name: &str) -> Option<&'static str> {
        match command_name {
            "create" => Some(
                "  create                   Create server with auto name/port\n  \
                 create <name>             Create server with custom name\n  \
                 create <name> <port>      Create with name and port\n  \
                 create <count>            Bulk create (1-100 servers)\n  \
                 create <name> <port> <n>  Bulk with base name, port, count\n\n  \
                 Examples:\n    \
                 create                    -> rss-001 on next free port\n    \
                 create mysite             -> mysite on next free port\n    \
                 create mysite 8080        -> mysite on port 8080\n    \
                 create 50                 -> 50 servers (rss-001..rss-050)\n    \
                 create web 8001 10        -> web-001:8001 .. web-010:8010",
            ),
            "start" => Some(
                "  start <id|name|number>   Start a single server\n  \
                 start <start>-<end>       Start range of servers\n  \
                 start all                 Start all stopped servers\n  \
                 --workers N, -w N         Set workers per server (1-16)\n\n  \
                 Note: 'start all' and ranges skip browser opening.\n  \
                 Bulk operations show time + memory benchmarks.\n\n  \
                 Examples:\n    \
                 start rss-001             -> start by name\n    \
                 start 1                   -> start server #1\n    \
                 start 1-10               -> start servers 1 through 10\n    \
                 start all                 -> start all stopped servers\n    \
                 start rss-001 --workers 3 -> start with 3 workers\n    \
                 start all -w 2            -> start all with 2 workers each",
            ),
            "stop" => Some(
                "  stop <id|name|number>    Stop a single server\n  \
                 stop <start>-<end>        Stop range of servers\n  \
                 stop all                  Stop all running servers\n\n  \
                 Examples:\n    \
                 stop rss-001              -> stop by name\n    \
                 stop 1                    -> stop server #1\n    \
                 stop 1-5                  -> stop servers 1 through 5\n    \
                 stop all                  -> stop all running servers",
            ),
            "list" => Some(
                "  list                     Show all servers (sorted by port)\n\n  \
                 Filter:\n    \
                 list running              Only running servers\n    \
                 list stopped              Only stopped servers\n    \
                 list failed               Only failed servers\n\n  \
                 Sort:\n    \
                 list -port asc            By port ascending (default)\n    \
                 list -port desc           By port descending\n    \
                 list -name asc            By name A-Z\n    \
                 list -name desc           By name Z-A\n\n  \
                 Special:\n    \
                 list memory               Disk + RAM usage per server\n\n  \
                 Combine: list running -name asc\n  \
                 Aliases: list servers, list server",
            ),
            "restart" => Some(
                "  restart                  Restart application (with confirm)\n  \
                 restart -f, --force       Force restart without confirm\n  \
                 restart -h, --help        Show help\n\n  \
                 Aliases: reboot, reset",
            ),
            "cleanup" => Some(
                "  cleanup                  Clean stopped servers (confirm)\n  \
                 cleanup stopped           Clean stopped servers\n  \
                 cleanup failed            Clean failed servers\n  \
                 cleanup logs              Clean all log files\n  \
                 cleanup www               Clean all www directories\n  \
                 cleanup www <name>        Clean specific server www\n  \
                 cleanup all               Clean everything\n  \
                 cleanup --force-stopped   Skip confirmation\n  \
                 cleanup --force-failed    Skip confirmation\n  \
                 cleanup --force-logs      Skip confirmation\n  \
                 cleanup --force-www       Skip confirmation\n  \
                 cleanup --force-all       Skip confirmation",
            ),
            "recover" => Some(
                "  recover                  Auto-fix inconsistent servers\n  \
                 recover all               Fix all servers\n  \
                 recover <id>              Fix specific server\n\n  \
                 Aliases: fix, status-fix",
            ),
            "remote" => Some(
                "  remote list              List SSH profiles\n  \
                 remote add <name> <user@host> <path> [port] [key]\n  \
                 remote show <name>        Show profile details\n  \
                 remote remove <name>      Delete profile\n  \
                 remote test <name>        Test SSH connection\n  \
                 remote help               Show help\n\n  \
                 Aliases: remote ls, remote rm, remote delete",
            ),
            "sync" => Some(
                "  sync push <remote> [path] [--delete] [--dry-run]\n  \
                 sync pull <remote> [path] [--delete] [--dry-run]\n  \
                 sync test <remote>        Test connection\n  \
                 sync exec <remote> <cmd>  Run remote command\n  \
                 sync restart <remote>     Restart remote service\n  \
                 sync git-pull <remote>    Remote git pull\n\n  \
                 Flags:\n    \
                 --delete                  Remove files not in source\n    \
                 --dry-run, -n             Preview without applying",
            ),
            "theme" => Some(
                "  theme                    Show current & available themes\n  \
                 theme <name>              Switch theme (live)\n  \
                 theme preview <name>      Preview theme\n  \
                 theme debug <name>        Show theme details\n  \
                 theme -h, --help          Show help",
            ),
            "lang" | "language" => Some(
                "  lang                     Show current language\n  \
                 lang <code>               Switch language (en, de, fr...)",
            ),
            "log-level" => Some(
                "  log-level                Show current level\n  \
                 log-level <level>         Set level (trace/debug/info/warn/error)\n  \
                 log-level -h, --help      Show help",
            ),
            "history" => Some(
                "  history                  Show info\n  \
                 history -c, --clear       Clear with confirmation\n  \
                 history -fc, --force-clear  Force clear\n  \
                 history -h, --help        Show help",
            ),
            "version" => Some(
                "  version                  Show version info\n\n  \
                 Alias: ver",
            ),
            "clear" => Some(
                "  clear                    Clear screen\n\n  \
                 Alias: cls",
            ),
            "exit" => Some(
                "  exit                     Exit with confirmation\n\n  \
                 Alias: q",
            ),
            "help" => Some(
                "  help                     Show all commands\n  \
                 help <command>            Show command details\n  \
                 help -s, --simple         Simple list\n  \
                 help -d, --detailed       Detailed list\n\n  \
                 Tip: Use '<command> ?' for quick help (e.g. 'create ?')",
            ),
            _ => None,
        }
    }

    /// Look up the localized description for a command, falling back to the original
    fn get_localized_description(&self, command_name: &str, original_description: &str) -> String {
        let normalized_name = command_name.replace("-", "_");
        let description_key = format!("system.commands.{}.description", normalized_name);

        if crate::i18n::has_translation(&description_key) {
            get_command_translation(&description_key, &[])
        } else {
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
                self.get_fallback_category_name(category_key)
            };

            result.push_str(&format!("{}:\n", category_name));

            for (name, description) in &commands {
                // Show short usage hint next to description
                let usage_hint = match *name {
                    "create" => " (create [name] [port] [count])",
                    "start" => " (start <id|name|all|1-N> [-w N])",
                    "stop" => " (stop <id|name|all|1-N>)",
                    "cleanup" => " (cleanup [stopped|failed|logs|www|all])",
                    "sync" => " (sync push|pull|test|exec ...)",
                    "remote" => " (remote list|add|show|remove|test)",
                    "lang" | "language" => " (lang [code])",
                    "theme" => " (theme [name|preview|debug])",
                    "log-level" => " (log-level [level])",
                    _ => "",
                };

                result.push_str(&format!(
                    "  {:12} {}{}\n",
                    name, description, usage_hint
                ));
            }
            result.push('\n');
        }

        result.push_str("  Tip: '<command> ?' for details (e.g. 'create ?')\n");
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

            result.push_str(&format!("  {}\n", name.to_uppercase()));
            result.push_str(&format!("  {}\n", localized_description));

            if let Some(usage) = Self::get_command_usage(name) {
                result.push_str(&format!("\n{}\n", usage));
            }

            result.push_str("\n  ──────────────────────────────\n\n");
        }

        result
    }

    /// Show help for a specific command (with full usage details)
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

                let mut result = format!(
                    "\n  {} - {}\n",
                    name.to_uppercase(),
                    localized_description
                );

                if let Some(usage) = Self::get_command_usage(name) {
                    result.push_str(&format!("\n{}\n", usage));
                }

                return result;
            }
        }

        get_command_translation("system.commands.help.command_not_found", &[command_name])
    }
}
