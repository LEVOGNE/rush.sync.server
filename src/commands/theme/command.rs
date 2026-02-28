use super::ThemeSystem;
use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct ThemeCommand {
    theme_system: std::sync::Mutex<Option<ThemeSystem>>,
}

impl ThemeCommand {
    pub fn new() -> Self {
        Self {
            theme_system: std::sync::Mutex::new(None),
        }
    }

    fn get_or_init_theme_system(&self) -> Result<std::sync::MutexGuard<'_, Option<ThemeSystem>>> {
        let mut guard = self.theme_system.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
        if guard.is_none() {
            *guard = Some(ThemeSystem::load()?);
        }
        Ok(guard)
    }
}

impl Command for ThemeCommand {
    fn name(&self) -> &'static str {
        "theme"
    }

    fn description(&self) -> &'static str {
        "Change application theme (live update without restart, loaded from TOML)"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("theme")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        log::info!("ThemeCommand::execute_sync called with args: {:?}", args);

        let mut guard = match self.get_or_init_theme_system() {
            Ok(guard) => {
                log::info!("ThemeSystem loaded successfully");
                guard
            }
            Err(e) => {
                log::error!("ThemeSystem load failed: {}", e);
                return Ok(format!(
                    "{} {}\n\n{}",
                    get_command_translation("system.commands.theme.load_failed", &[&e.to_string()]),
                    get_command_translation("system.commands.theme.no_themes_hint", &[]),
                    get_command_translation("system.commands.theme.add_sections_hint", &[])
                ));
            }
        };

        let theme_system = guard.as_mut().unwrap();

        match args.first() {
            None => {
                log::info!("Calling theme_system.show_status()");
                let result = theme_system.show_status_i18n();
                log::info!("show_status result: '{}'", result);
                Ok(result)
            }
            Some(&"--help" | &"-h") => {
                log::info!("Calling create_help_text()");
                let result = Self::create_help_text_i18n(theme_system);
                log::info!(
                    "create_help_text result length: {} chars",
                    result.chars().count()
                );
                Ok(result)
            }
            Some(&"debug") => match args.get(1) {
                Some(&theme_name) => Ok(theme_system.debug_theme_details_i18n(theme_name)),
                None => Ok(get_command_translation(
                    "system.commands.theme.debug_missing_name",
                    &[],
                )),
            },
            Some(&"preview") => match args.get(1) {
                Some(&theme_name) => theme_system.preview_theme_i18n(theme_name),
                None => Ok(get_command_translation(
                    "system.commands.theme.preview_missing_name",
                    &[],
                )),
            },
            Some(&theme_name) => {
                log::info!("Calling change_theme({})", theme_name);
                theme_system.change_theme_i18n(theme_name)
            }
        }
    }

    fn priority(&self) -> u8 {
        65
    }
}

impl ThemeCommand {
    fn create_help_text_i18n(theme_system: &ThemeSystem) -> String {
        let available_themes = theme_system.get_available_names();

        if available_themes.is_empty() {
            return format!(
                "{}\n\n{}",
                get_command_translation("system.commands.theme.no_themes_available", &[]),
                get_command_translation("system.commands.theme.how_to_add_themes", &[])
            );
        }

        let themes_list = available_themes.join(", ");

        format!(
            "{}\n{}\n{}\n{}\n{}\n\n{}\n{}\n{}\n{}\n\n{}",
            get_command_translation("system.commands.theme.help.header", &[]),
            get_command_translation("system.commands.theme.help.show_themes", &[]),
            get_command_translation("system.commands.theme.help.select_theme", &[&themes_list]),
            get_command_translation("system.commands.theme.help.preview_theme", &[]),
            get_command_translation("system.commands.theme.help.show_help", &[]),
            get_command_translation("system.commands.theme.help.live_loaded", &[]),
            get_command_translation("system.commands.theme.help.cursor_config", &[]),
            get_command_translation("system.commands.theme.help.add_sections", &[]),
            get_command_translation("system.commands.theme.help.live_changes", &[]),
            get_command_translation("system.commands.theme.help.cursor_options", &[])
        )
    }
}

impl Default for ThemeCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeSystem {
    pub fn show_status_i18n(&self) -> String {
        if self.themes.is_empty() {
            return get_command_translation("system.commands.theme.no_themes_found", &[]);
        }

        let themes_list = self.themes.keys().cloned().collect::<Vec<_>>().join(", ");
        get_command_translation(
            "system.commands.theme.current_status",
            &[&self.current_name.to_uppercase(), &themes_list],
        )
    }

    pub fn change_theme_i18n(&mut self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if !self.themes.contains_key(&theme_name_lower) {
            return Ok(if self.themes.is_empty() {
                get_command_translation("system.commands.theme.no_themes_found", &[])
            } else {
                let available = self.themes.keys().cloned().collect::<Vec<_>>().join(", ");
                get_command_translation(
                    "system.commands.theme.not_found",
                    &[theme_name, &available],
                )
            });
        }

        self.current_name = theme_name_lower.clone();

        // Log cursor details
        if let Some(theme_def) = self.themes.get(&theme_name_lower) {
            log::info!(
                "Theme '{}': input_cursor='{}' ({}), output_cursor='{}' ({}), prefix='{}'",
                theme_name_lower.to_uppercase(),
                theme_def.input_cursor,
                theme_def.input_cursor_color,
                theme_def.output_cursor,
                theme_def.output_cursor_color,
                theme_def.input_cursor_prefix
            );
        }

        // Async save
        let name_clone = theme_name_lower.clone();
        let paths_clone = self.config_paths.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::save_current_theme_to_config(&paths_clone, &name_clone).await {
                log::error!("Failed to save theme: {}", e);
            }
        });

        Ok(format!(
            "__LIVE_THEME_UPDATE__{}__MESSAGE__{}",
            theme_name_lower,
            get_command_translation(
                "system.commands.theme.changed_success",
                &[&theme_name_lower.to_uppercase()]
            )
        ))
    }

    pub fn preview_theme_i18n(&self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if let Some(theme_def) = self.themes.get(&theme_name_lower) {
            Ok(get_command_translation(
                "system.commands.theme.preview_details",
                &[
                    &theme_name_lower.to_uppercase(),
                    &theme_def.input_text,
                    &theme_def.input_bg,
                    &theme_def.output_text,
                    &theme_def.output_bg,
                    &theme_def.input_cursor_prefix,
                    &theme_def.input_cursor_color,
                    &theme_def.input_cursor,
                    &theme_def.output_cursor,
                    &theme_def.output_cursor_color,
                    &theme_name_lower,
                ],
            ))
        } else {
            let available = self.themes.keys().cloned().collect::<Vec<_>>().join(", ");
            Ok(get_command_translation(
                "system.commands.theme.not_found",
                &[theme_name, &available],
            ))
        }
    }

    pub fn debug_theme_details_i18n(&self, theme_name: &str) -> String {
        if let Some(theme_def) = self.themes.get(&theme_name.to_lowercase()) {
            get_command_translation(
                "system.commands.theme.debug_details",
                &[
                    &theme_name.to_uppercase(),
                    &theme_def.input_text,
                    &theme_def.input_bg,
                    &theme_def.output_text,
                    &theme_def.output_bg,
                    &theme_def.input_cursor_prefix,
                    &theme_def.input_cursor_color,
                    &theme_def.input_cursor,
                    &theme_def.output_cursor,
                    &theme_def.output_cursor_color,
                ],
            )
        } else {
            get_command_translation("system.commands.theme.debug_not_found", &[theme_name])
        }
    }
}
