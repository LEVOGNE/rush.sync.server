// commands/lang/config.rs - CONFIG INTEGRATION

use super::persistence::LanguagePersistence;
use crate::core::config::Config;
use crate::core::prelude::*;

/// Config-Integration für Language Management
pub struct LanguageConfig;

impl LanguageConfig {
    /// Setzt Sprache in bestehender Config-Instanz
    pub async fn set_in_config(config: &mut Config, lang: &str) -> Result<()> {
        // ✅ 1. VALIDIERUNG
        if !crate::i18n::AVAILABLE_LANGUAGES.contains(&lang) {
            return Err(AppError::Validation(format!(
                "Ungültige Sprache: {}. Verfügbar: {:?}",
                lang,
                crate::i18n::AVAILABLE_LANGUAGES
            )));
        }

        // ✅ 2. CONFIG AKTUALISIEREN
        config.language = lang.to_lowercase();

        // ✅ 3. CONFIG SPEICHERN
        config.save().await?;

        // ✅ 4. i18n AKTUALISIEREN
        crate::i18n::set_language(lang)?;

        log::debug!("Language set in config: {}", lang.to_uppercase());
        Ok(())
    }

    /// Lädt Sprache aus Config und setzt i18n
    pub async fn load_and_apply_from_config(config: &Config) -> Result<()> {
        let lang = &config.language;

        // ✅ SETZE i18n basierend auf Config
        if let Err(e) = crate::i18n::set_language(lang) {
            log::warn!(
                "{}",
                get_translation("system.config.language_set_failed", &[&e.to_string()])
            );

            // ✅ FALLBACK auf DEFAULT
            let _ = crate::i18n::set_language(crate::i18n::DEFAULT_LANGUAGE);
        }

        Ok(())
    }

    /// Erstellt neue Config mit spezifischer Sprache
    pub async fn create_config_with_language(lang: &str) -> Result<Config> {
        let mut config = Config::default();
        config.language = lang.to_lowercase();

        // ✅ SOFORT SPEICHERN
        config.save().await?;

        Ok(config)
    }

    /// Synchronisiert Config-Datei mit aktueller i18n Sprache
    pub async fn sync_config_with_current_language() -> Result<()> {
        let current_lang = crate::i18n::get_current_language().to_lowercase();
        LanguagePersistence::save_to_config(&current_lang).await
    }

    /// Prüft ob Config und i18n synchron sind
    pub fn is_config_in_sync_with_i18n(config: &Config) -> bool {
        let config_lang = config.language.to_lowercase();
        let current_lang = crate::i18n::get_current_language().to_lowercase();
        config_lang == current_lang
    }

    /// Repariert Config falls sie nicht mit i18n synchron ist
    pub async fn repair_config_sync(config: &mut Config) -> Result<bool> {
        if !Self::is_config_in_sync_with_i18n(config) {
            let current_lang = crate::i18n::get_current_language().to_lowercase();
            config.language = current_lang.clone();
            config.save().await?;

            log::debug!(
                "Config language repaired to: {}",
                current_lang.to_uppercase()
            );
            return Ok(true);
        }
        Ok(false)
    }
}
