// commands/lang/manager.rs - MIT FEHLENDER METHODE

use super::persistence::LanguagePersistence;
use crate::core::prelude::*;
use crate::i18n::{
    get_available_languages, get_command_translation, get_current_language, set_language,
};

/// Zentrale Verwaltung aller Language-Operationen
pub struct LanguageManager;

impl LanguageManager {
    /// Zeigt aktuellen Status und verfügbare Sprachen
    pub fn show_status() -> String {
        let current_lang = get_current_language();
        let available_langs = get_available_languages().join(", ");

        let current = get_command_translation("system.commands.language.current", &[&current_lang]);
        let available =
            get_command_translation("system.commands.language.available", &[&available_langs]);

        format!("{}\n{}", current, available)
    }

    /// Ändert die Sprache komplett (i18n + Config + Persistence)
    pub async fn change_language(lang: &str) -> Result<String> {
        // ✅ 1. VALIDIERUNG + i18n setzen
        match set_language(lang) {
            Ok(()) => {
                // ✅ 2. CONFIG PERSISTIEREN
                if let Err(e) = LanguagePersistence::save_to_config(lang).await {
                    log::error!("Failed to save language config: {}", e);
                    // Trotzdem Success, da i18n gesetzt wurde
                }

                // ✅ 3. SUCCESS MESSAGE (in neuer Sprache!)
                Ok(Self::create_save_message(
                    lang,
                    &get_command_translation(
                        "system.commands.language.changed",
                        &[&lang.to_uppercase()],
                    ),
                ))
            }
            Err(e) => {
                // ✅ 4. ERROR MESSAGE
                Ok(get_command_translation(
                    "system.commands.language.invalid",
                    &[&e.to_string()],
                ))
            }
        }
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

        // ✅ CACHE wird automatisch in set_language geleert
        if let Err(e) = crate::i18n::set_language(&lang_part) {
            return Some(format!("Fehler beim Setzen der Sprache: {}", e));
        }

        // ✅ CONFIG SPEICHERN
        if let Err(e) = LanguagePersistence::save_to_config(&lang_part).await {
            log::error!("Failed to save language config: {}", e);
        }

        Some(display_message.to_string())
    }

    /// Erstellt das spezielle Save-Message Format für screen.rs
    fn create_save_message(lang: &str, display_text: &str) -> String {
        format!("__SAVE_LANGUAGE__{}__MESSAGE__{}", lang, display_text)
    }

    /// ✅ FEHLENDE METHODE: Erstellt Save Message Format (public für command.rs)
    pub fn create_save_message_format(lang: &str, display_text: &str) -> String {
        format!("__SAVE_LANGUAGE__{}__MESSAGE__{}", lang, display_text)
    }

    /// Direkter Language-Switch ohne Config-Save (für interne Nutzung)
    pub fn switch_language_only(lang: &str) -> Result<()> {
        set_language(lang)
    }

    /// Gibt verfügbare Sprachen zurück
    pub fn get_available() -> Vec<String> {
        get_available_languages()
    }

    /// Gibt aktuelle Sprache zurück
    pub fn get_current() -> String {
        get_current_language()
    }
}
