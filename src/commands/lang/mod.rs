// =====================================================
// FILE: src/commands/lang/mod.rs - VEREINFACHTES LANGUAGE-SYSTEM
// =====================================================

use crate::core::prelude::*;
use crate::i18n::{get_available_languages, get_current_language, set_language};

pub mod command;
pub use command::LanguageCommand;

// ✅ VEREINFACHT: Alle Language-Logik in einer Struktur
#[derive(Debug)] // ✅ NEU
pub struct LanguageService {
    config_paths: Vec<std::path::PathBuf>,
}

impl LanguageService {
    /// Erstellt neuen LanguageService
    pub fn new() -> Self {
        Self {
            config_paths: crate::setup::setup_toml::get_config_paths(),
        }
    }

    /// Zeigt aktuellen Status und verfügbare Sprachen
    pub fn show_status(&self) -> String {
        let current_lang = get_current_language();
        let available_langs = get_available_languages().join(", ");

        let current = crate::i18n::get_command_translation(
            "system.commands.language.current",
            &[&current_lang],
        );
        let available = crate::i18n::get_command_translation(
            "system.commands.language.available",
            &[&available_langs],
        );

        format!("{}\n{}", current, available)
    }

    /// Ändert die Sprache komplett (i18n + Config + Persistence)
    pub async fn change_language(&mut self, lang: &str) -> Result<String> {
        // ✅ 1. VALIDIERUNG + i18n setzen
        match set_language(lang) {
            Ok(()) => {
                // ✅ 2. CONFIG PERSISTIEREN
                if let Err(e) = self.save_to_config(lang).await {
                    log::error!("Failed to save language config: {}", e);
                    // Trotzdem Success, da i18n gesetzt wurde
                }

                // ✅ 3. SUCCESS MESSAGE (in neuer Sprache!)
                Ok(self.create_save_message(
                    lang,
                    &crate::i18n::get_command_translation(
                        "system.commands.language.changed",
                        &[&lang.to_uppercase()],
                    ),
                ))
            }
            Err(e) => {
                // ✅ 4. ERROR MESSAGE
                Ok(crate::i18n::get_command_translation(
                    "system.commands.language.invalid",
                    &[&e.to_string()],
                ))
            }
        }
    }

    /// Direkter Language-Switch ohne Config-Save (für sync calls)
    pub fn switch_language_only(&self, lang: &str) -> Result<()> {
        set_language(lang)
    }

    /// Verarbeitet __SAVE_LANGUAGE__ Messages von screen.rs
    pub async fn process_save_message(message: &str) -> Option<String> {
        if !message.starts_with("__SAVE_LANGUAGE__") {
            return None;
        }

        let parts: Vec<&str> = message.split("__MESSAGE__").collect();
        if parts.len() != 2 {
            return None;
        }

        let lang_part = parts[0].replace("__SAVE_LANGUAGE__", "");
        let display_message = parts[1];

        // ✅ CONFIG SPEICHERN
        let service = LanguageService::new();
        if let Err(e) = service.save_to_config(&lang_part).await {
            log::error!("Failed to save language config: {}", e);
        }

        Some(display_message.to_string())
    }

    /// Gibt verfügbare Sprachen zurück
    pub fn get_available(&self) -> Vec<String> {
        get_available_languages()
    }

    /// Gibt aktuelle Sprache zurück
    pub fn get_current(&self) -> String {
        get_current_language()
    }

    // ✅ PRIVATE HELPERS

    /// Erstellt das spezielle Save-Message Format für screen.rs
    fn create_save_message(&self, lang: &str, display_text: &str) -> String {
        format!("__SAVE_LANGUAGE__{}__MESSAGE__{}", lang, display_text)
    }

    /// Speichert Sprache in Config-Datei
    async fn save_to_config(&self, lang: &str) -> Result<()> {
        for path in &self.config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(AppError::Io)?;
                let updated_content = self.update_language_in_toml(&content, lang)?;
                tokio::fs::write(path, updated_content)
                    .await
                    .map_err(AppError::Io)?;
                log::debug!("Language '{}' saved to config", lang.to_uppercase());
                return Ok(());
            }
        }
        Ok(())
    }

    /// Updated language.current in TOML-Inhalt
    fn update_language_in_toml(&self, content: &str, lang: &str) -> Result<String> {
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

        Ok(updated_content)
    }

    /// Lädt Sprache aus Config beim Startup
    pub async fn load_from_config(&self) -> Option<String> {
        for path in &self.config_paths {
            if path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    if let Some(lang) = self.extract_language_from_toml(&content) {
                        return Some(lang);
                    }
                }
            }
        }
        None
    }

    /// Extrahiert language.current aus TOML String
    fn extract_language_from_toml(&self, content: &str) -> Option<String> {
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

    /// ✅ NEU: Ersatz für LanguageConfig::load_and_apply_from_config
    pub async fn load_and_apply_from_config(
        &self,
        config: &crate::core::config::Config,
    ) -> Result<()> {
        let lang = &config.language;

        // ✅ SETZE i18n basierend auf Config
        if let Err(e) = crate::i18n::set_language(lang) {
            log::warn!(
                "{}",
                crate::i18n::get_translation(
                    "system.config.language_set_failed",
                    &[&e.to_string()]
                )
            );

            // ✅ FALLBACK auf DEFAULT
            let _ = crate::i18n::set_language(crate::i18n::DEFAULT_LANGUAGE);
        }

        Ok(())
    }
}

impl Default for LanguageService {
    fn default() -> Self {
        Self::new()
    }
}
