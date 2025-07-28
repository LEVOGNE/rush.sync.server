// =====================================================
// FILE: src/commands/theme/config.rs - KORRIGIERT
// =====================================================

use super::themes::{PredefinedThemes, ThemeDefinition};
use crate::core::prelude::*;

/// Config-Integration für Theme Management
pub struct ThemeConfig;

impl ThemeConfig {
    /// Speichert Theme in Config-Datei
    pub async fn save_theme_to_config(
        theme_name: &str,
        _theme_def: &ThemeDefinition,
    ) -> Result<()> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;

                let updated_content = Self::update_toml_with_theme(&content, theme_name)?;

                tokio::fs::write(&path, updated_content)
                    .await
                    .map_err(AppError::Io)?;

                log::debug!("Theme '{}' saved to config", theme_name.to_uppercase());
                return Ok(());
            }
        }
        Ok(())
    }

    /// Erweitert TOML um Theme-Support
    fn update_toml_with_theme(original_content: &str, current_theme: &str) -> Result<String> {
        let mut lines: Vec<String> = original_content.lines().map(|l| l.to_string()).collect();

        // ✅ 1. SETZE current_theme in [general]
        Self::set_current_theme_in_general(&mut lines, current_theme);

        // ✅ 2. FÜGE Theme-Definitionen hinzu (falls nicht vorhanden)
        Self::ensure_theme_definitions(&mut lines);

        Ok(lines.join("\n"))
    }

    /// Setzt current_theme in [general] Sektion
    fn set_current_theme_in_general(lines: &mut Vec<String>, current_theme: &str) {
        let mut in_general_section = false;
        let mut current_theme_updated = false;

        for line in lines.iter_mut() {
            let trimmed = line.trim();

            if trimmed == "[general]" {
                in_general_section = true;
            } else if in_general_section {
                if trimmed.starts_with("current_theme") {
                    *line = format!("current_theme = \"{}\"", current_theme);
                    current_theme_updated = true;
                } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_general_section = false;
                }
            }
        }

        // Falls current_theme nicht existiert, füge es zur [general] Sektion hinzu
        if !current_theme_updated {
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[general]" {
                    // Finde das Ende der [general] Sektion
                    let mut insert_index = lines.len();
                    for j in (i + 1)..lines.len() {
                        if lines[j].trim().starts_with('[') {
                            insert_index = j;
                            break;
                        }
                    }
                    lines.insert(
                        insert_index,
                        format!("current_theme = \"{}\"", current_theme),
                    );
                    break;
                }
            }
        }
    }

    /// Stellt sicher dass Theme-Definitionen vorhanden sind
    fn ensure_theme_definitions(lines: &mut Vec<String>) {
        // Prüfe ob bereits Theme-Definitionen vorhanden sind
        let has_theme_definitions = lines.iter().any(|line| line.trim().starts_with("[theme."));

        if !has_theme_definitions {
            lines.push("".to_string()); // Leerzeile

            // Füge alle predefined Themes hinzu
            let all_themes = PredefinedThemes::get_all();
            for (theme_name, theme_def) in all_themes.iter() {
                lines.push(format!("[theme.{}]", theme_name));
                lines.push(format!("input_text = \"{}\"", theme_def.input_text));
                lines.push(format!("input_bg = \"{}\"", theme_def.input_bg));
                lines.push(format!("cursor = \"{}\"", theme_def.cursor));
                lines.push(format!("output_text = \"{}\"", theme_def.output_text));
                lines.push(format!("output_bg = \"{}\"", theme_def.output_bg));
                lines.push("".to_string()); // Leerzeile
            }
        }
    }

    /// Lädt Theme aus Config
    pub async fn load_current_theme_from_config() -> Result<Option<String>> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;

                if let Some(theme) = Self::extract_current_theme_from_toml(&content) {
                    return Ok(Some(theme));
                }
            }
        }
        Ok(None)
    }

    /// PUBLIC: Extrahiert current_theme aus TOML
    pub fn extract_current_theme_from_toml(content: &str) -> Option<String> {
        let mut in_general_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[general]" {
                in_general_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[general]" {
                in_general_section = false;
                continue;
            }

            if in_general_section && trimmed.starts_with("current_theme =") {
                if let Some(value_part) = trimmed.split('=').nth(1) {
                    let cleaned = value_part.trim().trim_matches('"').trim_matches('\'');
                    return Some(cleaned.to_string());
                }
            }
        }
        None
    }
}
