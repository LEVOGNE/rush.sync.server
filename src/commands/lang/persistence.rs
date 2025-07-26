// commands/lang/persistence.rs - TOML FILE OPERATIONEN

use crate::core::prelude::*;
use std::path::PathBuf;

/// Verwaltet das Speichern/Laden von Language-Konfigurationen
pub struct LanguagePersistence;

impl LanguagePersistence {
    /// Speichert Sprache in Config-Datei (von screen.rs übernommen)
    pub async fn save_to_config(lang: &str) -> Result<()> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;

                // ✅ INTELLIGENT: Regex für saubere Ersetzung
                let updated_content = if content.contains("[language]") {
                    // Bestehende current = Zeile ersetzen
                    content
                        .lines()
                        .map(|line| {
                            if line.trim_start().starts_with("current =") {
                                format!("current = \"{}\"", lang)
                            } else {
                                line.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    // Language section hinzufügen
                    format!("{}\n\n[language]\ncurrent = \"{}\"", content.trim(), lang)
                };

                tokio::fs::write(&path, updated_content)
                    .await
                    .map_err(AppError::Io)?;

                log::debug!("Language '{}' saved to config", lang.to_uppercase());
                return Ok(());
            }
        }
        Ok(())
    }

    /// Lädt Sprache aus Config-Datei
    pub async fn load_from_config() -> Result<Option<String>> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;

                // ✅ PARSE TOML für language.current
                if let Some(lang) = Self::extract_language_from_toml(&content) {
                    return Ok(Some(lang));
                }
            }
        }
        Ok(None)
    }

    /// Extrahiert language.current aus TOML String
    fn extract_language_from_toml(content: &str) -> Option<String> {
        let mut in_language_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[language]" {
                in_language_section = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[language]" {
                in_language_section = false;
                continue;
            }

            if in_language_section && trimmed.starts_with("current =") {
                if let Some(value_part) = trimmed.split('=').nth(1) {
                    let cleaned = value_part.trim().trim_matches('"').trim_matches('\'');
                    return Some(cleaned.to_string());
                }
            }
        }
        None
    }

    /// Überprüft ob Config-Datei existiert
    pub fn config_exists() -> bool {
        crate::setup::setup_toml::get_config_paths()
            .iter()
            .any(|path| path.exists())
    }

    /// Gibt alle möglichen Config-Pfade zurück
    pub fn get_config_paths() -> Vec<PathBuf> {
        crate::setup::setup_toml::get_config_paths()
    }

    /// Erstellt Backup der aktuellen Config vor Language-Änderung
    pub async fn backup_config() -> Result<Option<PathBuf>> {
        let config_paths = Self::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let backup_path = path.with_extension("toml.backup");
                tokio::fs::copy(&path, &backup_path)
                    .await
                    .map_err(AppError::Io)?;

                log::debug!("Config backup created: {}", backup_path.display());
                return Ok(Some(backup_path));
            }
        }
        Ok(None)
    }
}
