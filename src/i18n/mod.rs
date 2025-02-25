// src/i18n/mod.rs
mod cache;
mod error;
mod langs;
mod service;
mod types;

use crate::core::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use crate::ui::color::ColorCategory;

pub use error::TranslationError;
pub(crate) use langs::{AVAILABLE_LANGUAGES, DEFAULT_LANGUAGE};
pub use types::{TranslationConfig, TranslationEntry};

use service::TranslationService;

pub async fn init() -> Result<()> {
    match ConfigHandler::new().await {
        Ok(config_handler) => {
            let lang = config_handler
                .get_setting("lang")
                .map(|l| l.to_lowercase())
                .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());

            set_language_internal(&lang, false)
        }
        Err(_) => set_language_internal(DEFAULT_LANGUAGE, false),
    }
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    TranslationService::get_instance()
        .write()
        .unwrap()
        .get_translation(key, params)
        .0
}

pub fn get_translation_details(key: &str) -> (String, ColorCategory) {
    TranslationService::get_instance()
        .write()
        .unwrap()
        .get_translation(key, &[])
}

pub fn set_language(lang: &str) -> Result<()> {
    set_language_internal(lang, true)
}

fn set_language_internal(lang: &str, save_config: bool) -> Result<()> {
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

    if save_config {
        tokio::spawn(async move {
            if let Ok(mut config_handler) = ConfigHandler::new().await {
                if let Err(e) = config_handler.set_setting("lang".to_string(), lang).await {
                    log::error!("Fehler beim Speichern der Spracheinstellung: {}", e);
                }
            }
        });
    }

    Ok(())
}

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
