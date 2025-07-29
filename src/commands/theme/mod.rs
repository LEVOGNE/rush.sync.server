// =====================================================
// FILE: src/commands/theme/mod.rs - VEREINFACHTES THEME-SYSTEM
// =====================================================

use crate::core::prelude::*;
use std::collections::HashMap;

pub mod command;
pub use command::ThemeCommand;

// ✅ VEREINFACHT: Alle Theme-Logik in einer Struktur
#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub input_text: String,
    pub input_bg: String,
    pub cursor: String,
    pub output_text: String,
    pub output_bg: String,
    pub prompt_text: String,
    pub prompt_color: String,
}

// ✅ HAUPTKLASSE: Alles was vorher auf 4 Module verteilt war
#[derive(Debug)] // ✅ NEU
pub struct ThemeSystem {
    themes: HashMap<String, ThemeDefinition>,
    current_name: String,
    config_paths: Vec<std::path::PathBuf>,
}

impl ThemeSystem {
    /// Lädt Theme-System aus TOML-Dateien
    pub fn load() -> Result<Self> {
        let config_paths = crate::setup::setup_toml::get_config_paths();
        let themes = Self::load_themes_from_paths(&config_paths)?;
        let current_name = Self::load_current_theme_name(&config_paths).unwrap_or_else(|| {
            // ✅ FALLBACK: Erstes verfügbares Theme
            themes
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string())
        });

        if themes.is_empty() {
            log::warn!("❌ Keine Themes in TOML gefunden! Füge [theme.xyz] Sektionen hinzu.");
        } else {
            log::info!(
                "✅ {} Themes aus TOML geladen: {}",
                themes.len(),
                themes.keys().cloned().collect::<Vec<String>>().join(", ")
            );
        }

        Ok(Self {
            themes,
            current_name,
            config_paths,
        })
    }

    /// Zeigt aktuellen Status
    pub fn show_status(&self) -> String {
        if self.themes.is_empty() {
            return "❌ Keine Themes verfügbar! Füge [theme.xyz] Sektionen zur rush.toml hinzu."
                .to_string();
        }

        let available: Vec<String> = self.themes.keys().cloned().collect(); // ✅ FIX: .cloned()
        format!(
            "Current theme: {} (aus TOML)\nVerfügbare Themes aus TOML: {}",
            self.current_name.to_uppercase(),
            available.join(", ")
        )
    }

    /// Live Theme Change mit TOML-Persistierung
    pub fn change_theme(&mut self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if self.themes.is_empty() {
            return Ok(
                "❌ Keine Themes verfügbar! Füge [theme.xyz] Sektionen zur rush.toml hinzu."
                    .to_string(),
            );
        }

        if !self.themes.contains_key(&theme_name_lower) {
            let available: Vec<String> = self.themes.keys().cloned().collect(); // ✅ FIX: .cloned()
            return Ok(format!(
                "❌ Theme '{}' nicht in TOML gefunden. Verfügbare TOML-Themes: {}",
                theme_name,
                available.join(", ")
            ));
        }

        // ✅ UPDATE current theme
        self.current_name = theme_name_lower.clone();

        // ✅ SAVE to config (background task)
        let theme_name_clone = theme_name_lower.clone();
        let config_paths = self.config_paths.clone();
        tokio::spawn(async move {
            if let Err(e) =
                Self::save_current_theme_to_config(&config_paths, &theme_name_clone).await
            {
                log::error!("Failed to save theme to config: {}", e);
            } else {
                log::info!(
                    "✅ TOML-Theme '{}' saved to config",
                    theme_name_clone.to_uppercase()
                );
            }
        });

        // ✅ LIVE UPDATE MESSAGE
        Ok(format!(
            "__LIVE_THEME_UPDATE__{}__MESSAGE__🎨 TOML-Theme changed to: {} ✨",
            theme_name_lower,
            theme_name_lower.to_uppercase()
        ))
    }

    /// Theme Preview
    pub fn preview_theme(&self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if self.themes.is_empty() {
            return Ok(
                "❌ Keine Themes verfügbar! Füge [theme.xyz] Sektionen zur rush.toml hinzu."
                    .to_string(),
            );
        }

        if let Some(theme_def) = self.themes.get(&theme_name_lower) {
            Ok(format!(
            "🎨 TOML-Theme '{}' Preview:\n  Input: {} auf {}\n  Output: {} auf {}\n  Cursor: {}\n  Prompt: '{}' in {}\n\n📁 Quelle: [theme.{}] in rush.toml",
            theme_name_lower.to_uppercase(),
            theme_def.input_text,
            theme_def.input_bg,
            theme_def.output_text,
            theme_def.output_bg,
            theme_def.cursor,
            theme_def.prompt_text,
            theme_def.prompt_color,
            theme_name_lower
        ))
        } else {
            let available: Vec<String> = self.themes.keys().cloned().collect(); // ✅ FIX: .cloned()
            Ok(format!(
                "❌ TOML-Theme '{}' nicht gefunden. Verfügbare: {}",
                theme_name,
                available.join(", ")
            ))
        }
    }

    /// Prüft ob Theme existiert
    pub fn theme_exists(&self, theme_name: &str) -> bool {
        self.themes.contains_key(&theme_name.to_lowercase())
    }

    /// Gibt Theme-Definition zurück
    pub fn get_theme(&self, theme_name: &str) -> Option<&ThemeDefinition> {
        self.themes.get(&theme_name.to_lowercase())
    }

    /// Verfügbare Theme-Namen
    pub fn get_available_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Aktueller Theme-Name
    pub fn get_current_name(&self) -> &str {
        &self.current_name
    }

    // ✅ PRIVATE HELPERS

    /// Lädt alle Themes aus TOML-Dateien
    fn load_themes_from_paths(
        config_paths: &[std::path::PathBuf],
    ) -> Result<HashMap<String, ThemeDefinition>> {
        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(themes) = Self::parse_themes_from_toml(&content) {
                        log::debug!(
                            "✅ {} TOML-Themes geladen aus: {}",
                            themes.len(),
                            themes.keys().cloned().collect::<Vec<String>>().join(", ") // ✅ FIX: .cloned() hinzugefügt
                        );
                        return Ok(themes);
                    }
                }
            }
        }

        log::warn!("❌ Keine TOML-Themes gefunden! Erstelle [theme.xyz] Sektionen.");
        Ok(HashMap::new())
    }

    /// Parst [theme.xyz] Sektionen aus TOML
    fn parse_themes_from_toml(content: &str) -> Result<HashMap<String, ThemeDefinition>> {
        let mut themes = HashMap::new();
        let mut current_theme_name: Option<String> = None;
        let mut current_theme_data: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Theme Section: [theme.dark]
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
            // Andere Section
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
            // Theme Property
            else if current_theme_name.is_some() && trimmed.contains('=') {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let clean_key = key.trim().to_string();
                    let clean_value = value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    if !clean_value.is_empty() {
                        current_theme_data.insert(clean_key, clean_value);
                    }
                }
            }
        }

        // Letztes Theme speichern
        if let Some(theme_name) = current_theme_name {
            if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                themes.insert(theme_name, theme_def);
            }
        }

        Ok(themes)
    }

    /// Baut ThemeDefinition aus geparsten Daten
    fn build_theme_from_data(data: &HashMap<String, String>) -> Option<ThemeDefinition> {
        Some(ThemeDefinition {
            input_text: data.get("input_text")?.clone(),
            input_bg: data.get("input_bg")?.clone(),
            cursor: data.get("cursor")?.clone(),
            output_text: data.get("output_text")?.clone(),
            output_bg: data.get("output_bg")?.clone(),
            prompt_text: data
                .get("prompt_text") // ✅ NEU
                .unwrap_or(&"/// ".to_string())
                .clone(),
            prompt_color: data
                .get("prompt_color") // ✅ NEU
                .unwrap_or(&"LightBlue".to_string())
                .clone(),
        })
    }

    /// Lädt current_theme aus TOML
    fn load_current_theme_name(config_paths: &[std::path::PathBuf]) -> Option<String> {
        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(theme) = Self::extract_current_theme_from_toml(&content) {
                        return Some(theme);
                    }
                }
            }
        }
        None
    }

    /// Extrahiert current_theme aus TOML
    fn extract_current_theme_from_toml(content: &str) -> Option<String> {
        let mut in_general_section = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed == "[general]" {
                in_general_section = true;
            } else if trimmed.starts_with('[') && trimmed != "[general]" {
                in_general_section = false;
            } else if in_general_section && trimmed.starts_with("current_theme") {
                if let Some(value_part) = trimmed.split('=').nth(1) {
                    let cleaned = value_part.trim().trim_matches('"').trim_matches('\'');
                    if !cleaned.is_empty() {
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
        None
    }

    /// Speichert current_theme in TOML-Config
    async fn save_current_theme_to_config(
        config_paths: &[std::path::PathBuf],
        theme_name: &str,
    ) -> Result<()> {
        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(AppError::Io)?;
                let updated_content = Self::update_current_theme_in_toml(&content, theme_name)?;
                tokio::fs::write(path, updated_content)
                    .await
                    .map_err(AppError::Io)?;
                return Ok(());
            }
        }
        Err(AppError::Validation("No config file found".to_string()))
    }

    /// Updated current_theme in TOML-Inhalt
    fn update_current_theme_in_toml(content: &str, theme_name: &str) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut in_general_section = false;
        let mut theme_updated = false;

        for line in lines.iter_mut() {
            let trimmed = line.trim();

            if trimmed == "[general]" {
                in_general_section = true;
            } else if trimmed.starts_with('[') && trimmed != "[general]" {
                in_general_section = false;
            } else if in_general_section && trimmed.starts_with("current_theme") {
                *line = format!("current_theme = \"{}\"", theme_name);
                theme_updated = true;
            }
        }

        if !theme_updated {
            // ✅ CLIPPY FIX: Iterator statt needless_range_loop
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[general]" {
                    // Finde Insert-Position
                    let insert_index = lines
                        .iter()
                        .enumerate()
                        .skip(i + 1) // ✅ Skip zur nächsten Position nach [general]
                        .find(|(_, line)| line.trim().starts_with('['))
                        .map(|(idx, _)| idx)
                        .unwrap_or(lines.len()); // ✅ Fallback: Am Ende einfügen

                    lines.insert(insert_index, format!("current_theme = \"{}\"", theme_name));
                    break;
                }
            }
        }

        Ok(lines.join("\n"))
    }
}
