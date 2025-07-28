use crate::core::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub input_text: String,
    pub input_bg: String,
    pub cursor: String,
    pub output_text: String,
    pub output_bg: String,
}

/// ✅ TOML-Theme-Loader - Lädt Themes aus Config-Datei
pub struct TomlThemeLoader;

impl TomlThemeLoader {
    /// Lädt alle Themes aus TOML-Datei
    pub async fn load_all_themes() -> Result<HashMap<String, ThemeDefinition>> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        if let Ok(themes) = Self::parse_themes_from_toml(&content) {
                            log::debug!(
                                "Loaded {} themes from TOML: {}",
                                themes.len(),
                                path.display()
                            );
                            return Ok(themes);
                        }
                    }
                    Err(e) => {
                        log::debug!("Could not read config file '{}': {}", path.display(), e);
                        continue;
                    }
                }
            }
        }

        // ✅ FALLBACK: Hardcodierte Themes falls keine TOML
        log::debug!("No TOML themes found, using fallback themes");
        Ok(Self::get_fallback_themes())
    }

    /// Lädt ein spezifisches Theme aus TOML
    pub async fn load_theme_by_name(theme_name: &str) -> Result<Option<ThemeDefinition>> {
        let all_themes = Self::load_all_themes().await?;
        Ok(all_themes.get(&theme_name.to_lowercase()).cloned())
    }

    /// Synchrone Version für Live-Updates (cached)
    pub fn load_theme_by_name_sync(theme_name: &str) -> Option<ThemeDefinition> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(themes) = Self::parse_themes_from_toml(&content) {
                        return themes.get(&theme_name.to_lowercase()).cloned();
                    }
                }
            }
        }

        // ✅ FALLBACK: Hardcodiert
        Self::get_fallback_themes()
            .get(&theme_name.to_lowercase())
            .cloned()
    }

    /// Parst [theme.xyz] Sektionen aus TOML-String
    pub fn parse_themes_from_toml(content: &str) -> Result<HashMap<String, ThemeDefinition>> {
        let mut themes = HashMap::new();
        let mut current_theme_name: Option<String> = None;
        let mut current_theme_data: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // ✅ IGNORE comments und empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // ✅ THEME SECTION: [theme.dark]
            if trimmed.starts_with("[theme.") && trimmed.ends_with(']') {
                // Speichere vorheriges Theme
                if let Some(theme_name) = current_theme_name.take() {
                    if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                        themes.insert(theme_name, theme_def);
                    }
                    current_theme_data.clear();
                }

                // Extrahiere neuen Theme-Namen
                if let Some(name) = trimmed
                    .strip_prefix("[theme.")
                    .and_then(|s| s.strip_suffix(']'))
                {
                    current_theme_name = Some(name.to_lowercase());
                }
            }
            // ✅ ANDERE SECTION: Speichere aktuelles Theme
            else if trimmed.starts_with('[')
                && trimmed.ends_with(']')
                && !trimmed.starts_with("[theme.")
            {
                if let Some(theme_name) = current_theme_name.take() {
                    if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                        themes.insert(theme_name, theme_def);
                    }
                    current_theme_data.clear();
                }
            }
            // ✅ THEME PROPERTY: input_text = "White"
            else if current_theme_name.is_some() && trimmed.contains('=') {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let clean_key = key.trim().to_string();
                    let clean_value = value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim()
                        .to_string();

                    if !clean_value.is_empty() {
                        current_theme_data.insert(clean_key, clean_value);
                    }
                }
            }
        }

        // ✅ LETZTES THEME speichern
        if let Some(theme_name) = current_theme_name {
            if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                themes.insert(theme_name, theme_def);
            }
        }

        log::debug!("Parsed {} themes from TOML", themes.len());
        Ok(themes)
    }

    /// Baut ThemeDefinition aus geparsten Daten
    pub fn build_theme_from_data(data: &HashMap<String, String>) -> Option<ThemeDefinition> {
        // ✅ ALLE REQUIRED FIELDS müssen vorhanden sein
        let input_text = data.get("input_text")?.clone();
        let input_bg = data.get("input_bg")?.clone();
        let cursor = data.get("cursor")?.clone();
        let output_text = data.get("output_text")?.clone();
        let output_bg = data.get("output_bg")?.clone();

        Some(ThemeDefinition {
            input_text,
            input_bg,
            cursor,
            output_text,
            output_bg,
        })
    }

    /// Fallback Themes (falls TOML nicht existiert)
    pub fn get_fallback_themes() -> HashMap<String, ThemeDefinition> {
        let mut themes = HashMap::new();

        themes.insert(
            "dark".to_string(),
            ThemeDefinition {
                input_text: "White".to_string(),
                input_bg: "Black".to_string(),
                cursor: "White".to_string(),
                output_text: "White".to_string(),
                output_bg: "Black".to_string(),
            },
        );

        themes.insert(
            "light".to_string(),
            ThemeDefinition {
                input_text: "Black".to_string(),
                input_bg: "White".to_string(),
                cursor: "Black".to_string(),
                output_text: "Black".to_string(),
                output_bg: "White".to_string(),
            },
        );

        themes.insert(
            "matrix".to_string(),
            ThemeDefinition {
                input_text: "LightGreen".to_string(),
                input_bg: "Black".to_string(),
                cursor: "LightGreen".to_string(),
                output_text: "Green".to_string(),
                output_bg: "Black".to_string(),
            },
        );

        themes.insert(
            "blue".to_string(),
            ThemeDefinition {
                input_text: "LightBlue".to_string(),
                input_bg: "Black".to_string(),
                cursor: "LightBlue".to_string(),
                output_text: "LightBlue".to_string(),
                output_bg: "Black".to_string(),
            },
        );

        themes
    }

    /// Prüft ob Theme in TOML existiert
    pub async fn theme_exists(theme_name: &str) -> bool {
        if let Ok(themes) = Self::load_all_themes().await {
            themes.contains_key(&theme_name.to_lowercase())
        } else {
            false
        }
    }

    /// Sync version für theme_exists
    pub fn theme_exists_sync(theme_name: &str) -> bool {
        Self::load_theme_by_name_sync(theme_name).is_some()
    }

    /// Gibt verfügbare Theme-Namen zurück (aus TOML)
    pub async fn get_available_names() -> Vec<String> {
        if let Ok(themes) = Self::load_all_themes().await {
            let mut names: Vec<String> = themes.keys().cloned().collect();
            names.sort();
            names
        } else {
            vec![
                "dark".to_string(),
                "light".to_string(),
                "matrix".to_string(),
                "blue".to_string(),
            ]
        }
    }
}

/// ✅ LEGACY COMPATIBILITY: PredefinedThemes wird zu Wrapper
pub struct PredefinedThemes;

impl PredefinedThemes {
    /// ✅ WRAPPER: Verwendet jetzt TOML-Loader
    pub fn get_by_name(name: &str) -> Option<ThemeDefinition> {
        TomlThemeLoader::load_theme_by_name_sync(name)
    }

    /// ✅ WRAPPER: Verwendet jetzt TOML-Loader
    pub fn exists(name: &str) -> bool {
        TomlThemeLoader::theme_exists_sync(name)
    }

    /// ✅ WRAPPER: Verwendet jetzt TOML-Loader (async wrapper für sync call)
    pub fn get_all() -> HashMap<String, ThemeDefinition> {
        // ✅ SYNC FALLBACK - verwendet cached/sync version
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(themes) = TomlThemeLoader::parse_themes_from_toml(&content) {
                        return themes;
                    }
                }
            }
        }

        // Fallback
        TomlThemeLoader::get_fallback_themes()
    }

    /// ✅ WRAPPER: Get available names
    pub fn get_available_names() -> Vec<String> {
        Self::get_all().keys().cloned().collect()
    }
}
