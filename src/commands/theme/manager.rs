// =====================================================
// FILE: src/commands/theme/manager.rs - TOML-BASIERTE THEME VERWALTUNG
// =====================================================

use super::themes::TomlThemeLoader;
use crate::core::prelude::*;

pub struct ThemeManager;

impl ThemeManager {
    /// ✅ ZEIGT aktuellen Theme-Status (aus TOML)
    pub fn show_status() -> String {
        let current_theme =
            Self::get_current_theme_from_config().unwrap_or_else(|| "dark".to_string());

        // ✅ TOML-basierte verfügbare Themes (Runtime-Aufruf)
        let available_themes = match Self::get_available_themes_sync() {
            Ok(themes) => themes.join(", "),
            Err(_) => "dark, light, matrix, blue".to_string(), // Fallback
        };

        format!(
            "Current theme: {}\nAvailable themes: {}",
            current_theme.to_uppercase(),
            available_themes
        )
    }

    /// ✅ ASYNC LIVE THEME CHANGE - Lädt aus TOML, keine Hardcodierung!
    pub async fn change_theme(theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        // ✅ VALIDIERUNG: Prüfe gegen TOML-Themes
        if !TomlThemeLoader::theme_exists_sync(&theme_name_lower) {
            let available = TomlThemeLoader::get_available_names().await.join(", ");
            return Ok(format!(
                "❌ Invalid theme: '{}'. Available: {}",
                theme_name, available
            ));
        }

        // ✅ LADE aktuelle Config
        match crate::core::config::Config::load_with_messages(false).await {
            Ok(mut config) => {
                // ✅ UPDATE Theme in Config + speichern
                match config.change_theme(&theme_name_lower).await {
                    Ok(()) => {
                        log::info!(
                            "✅ Theme '{}' saved to config (loaded from TOML)",
                            theme_name_lower.to_uppercase()
                        );

                        // ✅ LIVE UPDATE MESSAGE (Screen Manager wird das verarbeiten)
                        Ok(format!(
                            "__LIVE_THEME_UPDATE__{}__MESSAGE__🎨 Theme changed to: {} (from TOML)",
                            theme_name_lower,
                            theme_name_lower.to_uppercase()
                        ))
                    }
                    Err(e) => {
                        log::error!("❌ Failed to save theme: {}", e);
                        Ok(format!("❌ Failed to save theme: {}", e))
                    }
                }
            }
            Err(e) => {
                log::error!("❌ Failed to load config: {}", e);
                Ok(format!("❌ Failed to load config: {}", e))
            }
        }
    }

    /// ✅ SYNC Version - Immediate feedback + background config save
    pub fn change_theme_sync(theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        // ✅ VALIDIERUNG: Prüfe gegen TOML-Themes
        if !TomlThemeLoader::theme_exists_sync(&theme_name_lower) {
            // ✅ SYNC VERSION: Lade verfügbare Themes aus TOML
            let available = match Self::get_available_themes_sync() {
                Ok(themes) => themes.join(", "),
                Err(_) => "dark, light, matrix, blue".to_string(),
            };

            return Ok(format!(
                "❌ Invalid theme: '{}'. Available: {}",
                theme_name, available
            ));
        }

        // ✅ SOFORTIGER Live-Update Message (kein Restart!)
        let live_update_msg = format!(
            "__LIVE_THEME_UPDATE__{}__MESSAGE__🎨 Theme changed to: {} (from TOML)",
            theme_name_lower,
            theme_name_lower.to_uppercase()
        );

        // ✅ BACKGROUND Task für Config-Persistierung
        let theme_name_clone = theme_name_lower.clone();
        tokio::spawn(async move {
            // ✅ Kurze Verzögerung für bessere UX
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            match crate::core::config::Config::load_with_messages(false).await {
                Ok(mut config) => match config.change_theme(&theme_name_clone).await {
                    Ok(()) => {
                        log::info!(
                            "✅ Theme '{}' persisted to config (background, from TOML)",
                            theme_name_clone.to_uppercase()
                        );
                    }
                    Err(e) => {
                        log::error!("❌ Background theme save failed: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("❌ Background config load failed: {}", e);
                }
            }
        });

        Ok(live_update_msg)
    }

    /// ✅ THEME PREVIEW (aus TOML)
    pub fn preview_theme(theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        // ✅ LADE THEME AUS TOML
        if let Some(theme_def) = TomlThemeLoader::load_theme_by_name_sync(&theme_name_lower) {
            Ok(format!(
                "🎨 Theme '{}' Preview (from TOML):\n  Input: {} on {}\n  Output: {} on {}\n  Cursor: {}",
                theme_name_lower.to_uppercase(),
                theme_def.input_text,
                theme_def.input_bg,
                theme_def.output_text,
                theme_def.output_bg,
                theme_def.cursor
            ))
        } else {
            let available = match Self::get_available_themes_sync() {
                Ok(themes) => themes.join(", "),
                Err(_) => "dark, light, matrix, blue".to_string(),
            };

            Ok(format!(
                "❌ Invalid theme: '{}'. Available: {}",
                theme_name, available
            ))
        }
    }

    /// ✅ HELPER: Lädt aktuelles Theme aus Config (robust + cached)
    fn get_current_theme_from_config() -> Option<String> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                // ✅ ROBUST: Fehler-Handling für jede Datei einzeln
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        if let Some(theme) = Self::extract_current_theme_from_toml(&content) {
                            return Some(theme);
                        }
                    }
                    Err(e) => {
                        log::debug!("Could not read config file '{}': {}", path.display(), e);
                        continue; // Versuche nächste Datei
                    }
                }
            }
        }

        // ✅ FALLBACK: Default theme falls keine Config gefunden
        log::debug!("No config file found, using default theme");
        Some("dark".to_string())
    }

    /// ✅ HELPER: Extrahiert current_theme aus TOML (robust)
    fn extract_current_theme_from_toml(content: &str) -> Option<String> {
        let mut in_general_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // ✅ IGNORE comments und empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed == "[general]" {
                in_general_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[general]" {
                in_general_section = false;
                continue;
            }

            if in_general_section && trimmed.starts_with("current_theme") {
                if let Some(value_part) = trimmed.split('=').nth(1) {
                    let cleaned = value_part
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim();

                    if !cleaned.is_empty() {
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
        None
    }

    /// ✅ HELPER: Verfügbare Themes aus TOML (sync version)
    fn get_available_themes_sync() -> Result<Vec<String>> {
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
            "green".to_string(),
            "blue".to_string(),
        ])
    }
}

// =====================================================
// TEST EXAMPLES:
// =====================================================

/*
// ✅ THEME AUS TOML LADEN:
let theme = TomlThemeLoader::load_theme_by_name_sync("dark");

// ✅ LIVE THEME CHANGE (aus TOML):
let result = ThemeManager::change_theme_sync("matrix").unwrap();
// Result: "__LIVE_THEME_UPDATE__matrix__MESSAGE__🎨 Theme changed to: MATRIX (from TOML)"

// ✅ THEME PREVIEW (aus TOML):
let preview = ThemeManager::preview_theme("blue").unwrap();
// Result: "🎨 Theme 'BLUE' Preview (from TOML): ..."

// ✅ VERFÜGBARE THEMES (aus TOML):
let status = ThemeManager::show_status();
// Result: "Current theme: DARK\nAvailable themes: blue, dark, light, matrix"
*/
