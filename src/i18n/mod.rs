// src/i18n/mod.rs - MIT ASCII-MARKERN
mod cache;
mod error;
mod langs;
mod service;
mod types;

use crate::core::prelude::*;
use crate::ui::color::AppColor;

pub use error::TranslationError;
pub(crate) use langs::{AVAILABLE_LANGUAGES, DEFAULT_LANGUAGE};
pub use types::{TranslationConfig, TranslationEntry};

use service::TranslationService;

// ✅ INIT-FUNKTION
pub async fn init() -> Result<()> {
    set_language_internal(DEFAULT_LANGUAGE, false)
}

fn set_language_internal(lang: &str, _save_config: bool) -> Result<()> {
    let lang = lang.to_lowercase();

    if !is_language_available(&lang) {
        return Err(AppError::Translation(TranslationError::InvalidLanguage(
            lang.to_uppercase(),
        )));
    }

    let config = match TranslationConfig::load(&lang) {
        Ok(cfg) => cfg,
        Err(e) => {
            log::warn!("Fehler beim Laden der Sprachkonfiguration: {}", e);
            TranslationConfig::default()
        }
    };

    let mut service = TranslationService::get_instance().write().unwrap();
    service.current_language = lang.clone();
    service.config = config;

    Ok(())
}

// ✅ HAUPTFUNKTION: Text + Farbe in einem Aufruf
pub fn get_translation_with_color(key: &str, params: &[&str]) -> (String, AppColor) {
    TranslationService::get_instance()
        .write()
        .unwrap()
        .get_translation(key, params)
}

// ✅ NUR TEXT (für normale Verwendung)
pub fn get_translation(key: &str, params: &[&str]) -> String {
    get_translation_with_color(key, params).0
}

// ✅ NUR FARBE (falls mal getrennt gebraucht)
pub fn get_translation_color(key: &str) -> AppColor {
    get_translation_with_color(key, &[]).1
}

// ✅ FERTIG FORMATIERTE NACHRICHT (für Logging)
pub fn get_colored_translation(key: &str, params: &[&str]) -> String {
    let (text, color) = get_translation_with_color(key, params);
    color.format_message("", &text)
}

// ✅ COMMAND-SYSTEM MIT ASCII-MARKERN - CRASH-SAFE!
pub fn get_command_translation(key: &str, params: &[&str]) -> String {
    let service = TranslationService::get_instance().write().unwrap();

    if let Some(entry) = service.config.get_entry(key) {
        let (text, _color) = entry.format(params);
        // ✅ ASCII-MARKER - Kein Unicode-Crash!
        format!("[{}] {}", entry.category, text)
    } else {
        // Fallback mit Warning-Marker
        format!("[warning] ⚠️ Translation key not found: {}", key)
    }
}

// ✅ SPRACHE SETZEN
pub fn set_language(lang: &str) -> Result<()> {
    set_language_internal(lang, true)
}

// ✅ HILFSFUNKTIONEN
fn is_language_available(lang: &str) -> bool {
    AVAILABLE_LANGUAGES.iter().any(|&l| l == lang)
}

pub fn get_current_language() -> String {
    TranslationService::get_instance()
        .read()
        .unwrap()
        .current_language
        .to_uppercase()
}

pub fn get_available_languages() -> Vec<String> {
    AVAILABLE_LANGUAGES
        .iter()
        .map(|&s| s.to_uppercase())
        .collect()
}

pub fn get_translation_stats() -> (usize, usize) {
    TranslationService::get_instance()
        .read()
        .unwrap()
        .cache
        .stats()
}

pub fn clear_translation_cache() {
    TranslationService::get_instance()
        .write()
        .unwrap()
        .cache
        .clear();
}
