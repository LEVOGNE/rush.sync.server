// =====================================================
// FILE: src/commands/theme/command.rs - TOML-BASIERTE COMMANDS
// =====================================================

use super::manager::ThemeManager;
use super::themes::TomlThemeLoader;
use crate::commands::command::Command;
use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct ThemeCommand;

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

    /// ✅ SYNC: Nutzt TOML-basierte change_theme_sync für immediate live update
    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => Ok(ThemeManager::show_status()),
            Some(&"--help" | &"-h") => Ok(Self::create_help_text()),
            Some(&"preview") => match args.get(1) {
                Some(&theme_name) => ThemeManager::preview_theme(theme_name),
                None => Ok("❌ Theme name missing. Usage: theme preview <name>".to_string()),
            },
            Some(&theme_name) => {
                // ✅ VALIDIERUNG: Prüfe gegen TOML-Themes (nicht hardcodiert!)
                if TomlThemeLoader::theme_exists_sync(theme_name) {
                    // ✅ SYNC VERSION: Sofortiges live update mit background save
                    ThemeManager::change_theme_sync(theme_name)
                } else {
                    // ✅ DYNAMISCHE FEHLERMELDUNG mit aktuellen TOML-Themes
                    let available = match Self::get_available_themes_for_error() {
                        Ok(themes) => themes.join(", "),
                        Err(_) => "dark, light, matrix, blue".to_string(),
                    };

                    Ok(format!(
                        "❌ Invalid theme: '{}'. Available themes in TOML: {}",
                        theme_name, available
                    ))
                }
            }
        }
    }

    /// ✅ ASYNC: Nutzt echte async version für saubere config-saves
    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            match args.first() {
                None => Ok(ThemeManager::show_status()),
                Some(&"--help" | &"-h") => Ok(Self::create_help_text()),
                Some(&"preview") => Ok(self.execute_sync(args)?),
                Some(&theme_name) => {
                    // ✅ ASYNC VALIDIERUNG: Prüfe gegen TOML-Themes
                    if TomlThemeLoader::theme_exists_sync(theme_name) {
                        // ✅ ASYNC VERSION: Saubere config-save dann live update
                        ThemeManager::change_theme(theme_name).await
                    } else {
                        // ✅ ASYNC FEHLERMELDUNG mit TOML-Themes
                        let available = TomlThemeLoader::get_available_names().await.join(", ");
                        Ok(format!(
                            "❌ Invalid theme: '{}'. Available themes in TOML: {}",
                            theme_name, available
                        ))
                    }
                }
            }
        })
    }

    fn supports_async(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        65
    }
}

impl ThemeCommand {
    /// ✅ HELPER: Erstellt dynamischen Help-Text mit aktuellen TOML-Themes
    fn create_help_text() -> String {
        let available_themes = match Self::get_available_themes_for_error() {
            Ok(themes) => themes.join(", "),
            Err(_) => "dark, light, matrix, blue".to_string(),
        };

        format!(
            "Theme Commands (Live Update - No Restart - Loaded from TOML!):\n\
            theme                Show available themes\n\
            theme <name>         Select theme: {}\n\
            theme preview <name> Preview theme colors\n\
            theme -h             Show this help\n\n\
            ✨ All theme changes apply instantly without restart!\n\
            📁 Themes are loaded from your rush.toml [theme.*] sections",
            available_themes
        )
    }

    /// ✅ HELPER: Verfügbare Themes für Fehlermeldungen (sync)
    fn get_available_themes_for_error() -> Result<Vec<String>> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(themes) = TomlThemeLoader::parse_themes_from_toml(&content) {
                        let mut names: Vec<String> = themes.keys().cloned().collect();
                        names.sort();
                        return Ok(names);
                    }
                }
            }
        }

        // ✅ FALLBACK
        Ok(vec![
            "dark".to_string(),
            "light".to_string(),
            "matrix".to_string(),
            "blue".to_string(),
        ])
    }
}

// =====================================================
// BEISPIEL VERWENDUNG:
// =====================================================

/*
// ✅ THEME COMMAND BEISPIELE:

// 1. Status anzeigen (aus TOML):
theme
// Output: "Current theme: DARK\nAvailable themes: blue, dark, light, matrix, custom1, custom2"

// 2. Theme wechseln (aus TOML):
theme matrix
// Output: "__LIVE_THEME_UPDATE__matrix__MESSAGE__🎨 Theme changed to: MATRIX (from TOML)"

// 3. Theme preview (aus TOML):
theme preview blue
// Output: "🎨 Theme 'BLUE' Preview (from TOML): ..."

// 4. Help mit aktuellen TOML-Themes:
theme -h
// Output: "Theme Commands...\ntheme <name>    Select theme: blue, dark, light, matrix, custom1"

// 5. Ungültiges Theme (dynamische Fehlermeldung):
theme invalid
// Output: "❌ Invalid theme: 'invalid'. Available themes in TOML: blue, dark, light, matrix"
*/
