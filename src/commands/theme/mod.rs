// ## FILE: src/commands/theme/mod.rs - MIT input_cursor SUPPORT
// ## BEGIN ##
use crate::core::prelude::*;
use std::collections::HashMap;

pub mod command;
pub use command::ThemeCommand;

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub input_text: String,
    pub input_bg: String,
    pub cursor: String,
    pub output_text: String,
    pub output_bg: String,

    // ✅ PERFEKTE CURSOR-KONFIGURATION (5 Felder)
    pub input_cursor_prefix: String, // NEU: Prompt-Text
    pub input_cursor_color: String,  // NEU: Prompt-Farbe
    pub input_cursor: String,        // NEU: Input-Cursor-Typ
    pub output_cursor: String,       // Output-Cursor-Typ
    pub output_cursor_color: String, // NEU: Output-Cursor-Farbe
}

#[derive(Debug)]
pub struct ThemeSystem {
    themes: HashMap<String, ThemeDefinition>,
    current_name: String,
    config_paths: Vec<std::path::PathBuf>,
}

impl ThemeSystem {
    pub fn load() -> Result<Self> {
        let config_paths = crate::setup::setup_toml::get_config_paths();
        let themes = Self::load_themes_from_paths(&config_paths)?;
        let current_name = Self::load_current_theme_name(&config_paths).unwrap_or_else(|| {
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

    pub fn show_status(&self) -> String {
        if self.themes.is_empty() {
            return "❌ Keine Themes verfügbar! Füge [theme.xyz] Sektionen zur rush.toml hinzu."
                .to_string();
        }

        let available: Vec<String> = self.themes.keys().cloned().collect();
        format!(
            "Current theme: {} (aus TOML)\nVerfügbare Themes aus TOML: {}",
            self.current_name.to_uppercase(),
            available.join(", ")
        )
    }

    pub fn change_theme(&mut self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if self.themes.is_empty() {
            return Ok(
                "❌ Keine Themes verfügbar! Füge [theme.xyz] Sektionen zur rush.toml hinzu."
                    .to_string(),
            );
        }

        if !self.themes.contains_key(&theme_name_lower) {
            let available: Vec<String> = self.themes.keys().cloned().collect();
            return Ok(format!(
                "❌ Theme '{}' nicht in TOML gefunden. Verfügbare TOML-Themes: {}",
                theme_name,
                available.join(", ")
            ));
        }

        self.current_name = theme_name_lower.clone();

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

        Ok(format!(
            "__LIVE_THEME_UPDATE__{}__MESSAGE__🎨 TOML-Theme changed to: {} ✨",
            theme_name_lower,
            theme_name_lower.to_uppercase()
        ))
    }

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
                "🎨 TOML-Theme '{}' Preview:\n  Input: {} auf {}\n  Output: {} auf {}\n  Cursor: {}\n  Input-Cursor-Prefix: '{}' in {} ✅ NEU!\n  Input-Cursor: {} ✅ NEU!\n  Output-Cursor: {} in {} ✅ NEU!\n\n📁 Quelle: [theme.{}] in rush.toml",
                theme_name_lower.to_uppercase(),
                theme_def.input_text,
                theme_def.input_bg,
                theme_def.output_text,
                theme_def.output_bg,
                theme_def.cursor,
                theme_def.input_cursor_prefix,
                theme_def.input_cursor_color,
                theme_def.input_cursor,
                theme_def.output_cursor,
                theme_def.output_cursor_color,
                theme_name_lower
            ))
        } else {
            let available: Vec<String> = self.themes.keys().cloned().collect();
            Ok(format!(
                "❌ TOML-Theme '{}' nicht gefunden. Verfügbare: {}",
                theme_name,
                available.join(", ")
            ))
        }
    }

    pub fn theme_exists(&self, theme_name: &str) -> bool {
        self.themes.contains_key(&theme_name.to_lowercase())
    }

    pub fn get_theme(&self, theme_name: &str) -> Option<&ThemeDefinition> {
        self.themes.get(&theme_name.to_lowercase())
    }

    pub fn get_available_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_current_name(&self) -> &str {
        &self.current_name
    }

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
                            themes.keys().cloned().collect::<Vec<String>>().join(", ")
                        );
                        return Ok(themes);
                    }
                }
            }
        }

        log::warn!("❌ Keine TOML-Themes gefunden! Erstelle [theme.xyz] Sektionen.");
        Ok(HashMap::new())
    }

    fn parse_themes_from_toml(content: &str) -> Result<HashMap<String, ThemeDefinition>> {
        let mut themes = HashMap::new();
        let mut current_theme_name: Option<String> = None;
        let mut current_theme_data: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with("[theme.") && trimmed.ends_with(']') {
                if let Some(theme_name) = current_theme_name.take() {
                    if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                        themes.insert(theme_name, theme_def);
                    }
                    current_theme_data.clear();
                }

                if let Some(name) = trimmed
                    .strip_prefix("[theme.")
                    .and_then(|s| s.strip_suffix(']'))
                {
                    current_theme_name = Some(name.to_lowercase());
                }
            } else if trimmed.starts_with('[')
                && trimmed.ends_with(']')
                && !trimmed.starts_with("[theme.")
            {
                if let Some(theme_name) = current_theme_name.take() {
                    if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                        themes.insert(theme_name, theme_def);
                    }
                    current_theme_data.clear();
                }
            } else if current_theme_name.is_some() && trimmed.contains('=') {
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

        if let Some(theme_name) = current_theme_name {
            if let Some(theme_def) = Self::build_theme_from_data(&current_theme_data) {
                themes.insert(theme_name, theme_def);
            }
        }

        Ok(themes)
    }

    fn build_theme_from_data(data: &HashMap<String, String>) -> Option<ThemeDefinition> {
        // ✅ BACKWARD-KOMPATIBILITÄT mit perfekter Struktur
        let input_cursor_prefix = data.get("input_cursor_prefix")
            .or_else(|| {
                if let Some(legacy) = data.get("prompt_text") {
                    log::warn!("⚠️ Veraltetes 'prompt_text' in Theme gefunden, verwende als 'input_cursor_prefix': {}", legacy);
                    Some(legacy)
                } else {
                    None
                }
            })
            .unwrap_or(&"/// ".to_string())
            .clone();

        let input_cursor_color = data.get("input_cursor_color")
            .or_else(|| {
                if let Some(legacy) = data.get("prompt_color") {
                    log::warn!("⚠️ Veraltetes 'prompt_color' in Theme gefunden, verwende als 'input_cursor_color': {}", legacy);
                    Some(legacy)
                } else {
                    None
                }
            })
            .unwrap_or(&"LightBlue".to_string())
            .clone();

        let input_cursor = data.get("input_cursor")
            .or_else(|| {
                if let Some(legacy) = data.get("prompt_cursor") {
                    log::warn!("⚠️ Veraltetes 'prompt_cursor' in Theme gefunden, verwende als 'input_cursor': {}", legacy);
                    Some(legacy)
                } else {
                    None
                }
            })
            .unwrap_or(&"DEFAULT".to_string())
            .clone();

        let output_cursor_color = data.get("output_cursor_color")
            .or_else(|| {
                if let Some(legacy) = data.get("output_color") {
                    log::warn!("⚠️ Veraltetes 'output_color' in Theme gefunden, verwende als 'output_cursor_color': {}", legacy);
                    Some(legacy)
                } else {
                    None
                }
            })
            .unwrap_or(&"White".to_string())
            .clone();

        Some(ThemeDefinition {
            input_text: data.get("input_text")?.clone(),
            input_bg: data.get("input_bg")?.clone(),
            cursor: data.get("cursor")?.clone(),
            output_text: data.get("output_text")?.clone(),
            output_bg: data.get("output_bg")?.clone(),

            // ✅ PERFEKTE CURSOR-KONFIGURATION
            input_cursor_prefix,
            input_cursor_color,
            input_cursor,
            output_cursor: data
                .get("output_cursor")
                .unwrap_or(&"DEFAULT".to_string())
                .clone(),
            output_cursor_color,
        })
    }

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
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[general]" {
                    let insert_index = lines
                        .iter()
                        .enumerate()
                        .skip(i + 1)
                        .find(|(_, line)| line.trim().starts_with('['))
                        .map(|(idx, _)| idx)
                        .unwrap_or(lines.len());

                    lines.insert(insert_index, format!("current_theme = \"{}\"", theme_name));
                    break;
                }
            }
        }

        Ok(lines.join("\n"))
    }
}
// ## END ##
