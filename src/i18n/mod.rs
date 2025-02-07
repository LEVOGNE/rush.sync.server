// src/i18n/mod.rs
use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Translations {
    pub system: SystemTranslations,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemTranslations {
    pub startup: StartupTranslations,
    pub commands: CommandTranslations,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StartupTranslations {
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandTranslations {
    pub unknown: String,
    pub language: LanguageTranslations,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LanguageTranslations {
    pub current: String,
    pub changed: String,
    pub invalid: String,
    pub available: String,
}

// Verfügbare Sprachen als Konstante
const AVAILABLE_LANGUAGES: [&str; 2] = ["de", "en"];

lazy_static! {
    static ref CURRENT_LANGUAGE: RwLock<String> = RwLock::new("de".to_string());
    static ref TRANSLATIONS: RwLock<Translations> =
        RwLock::new(load_translations().expect("Fehler beim Laden der Übersetzungen"));
}

pub fn load_translations() -> Result<Translations> {
    let current_lang = CURRENT_LANGUAGE.read().unwrap();
    let translations_str = match current_lang.as_str() {
        "de" => include_str!("trans_de.json"),
        "en" => include_str!("trans_en.json"),
        _ => include_str!("trans_de.json"), // Fallback auf Deutsch
    };

    serde_json::from_str(translations_str)
        .map_err(|e| AppError::Validation(format!("Fehler beim Parsen der Übersetzungen: {}", e)))
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    let translations = TRANSLATIONS.read().unwrap();
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["system", "startup", "version"] => {
            let template = &translations.system.startup.version;
            if let Some(&param) = params.first() {
                template.replace("{}", param)
            } else {
                template.to_string()
            }
        }
        ["system", "commands", "unknown"] => {
            // NEU: Handle unknown command translation
            let template = &translations.system.commands.unknown;
            if let Some(&param) = params.first() {
                template.replace("{}", param)
            } else {
                template.to_string()
            }
        }
        ["system", "commands", "language", subkey] => {
            let lang_translations = &translations.system.commands.language;
            let template = match *subkey {
                "current" => &lang_translations.current,
                "changed" => &lang_translations.changed,
                "invalid" => &lang_translations.invalid,
                "available" => &lang_translations.available,
                _ => return format!("Translation key not found: {}", key),
            };
            if let Some(&param) = params.first() {
                template.replace("{}", param)
            } else {
                template.to_string()
            }
        }
        _ => format!("Translation key not found: {}", key),
    }
}

pub async fn init_language_silent() -> Result<()> {
    let config_handler = ConfigHandler::new().await?;
    if let Some(saved_lang) = config_handler.get_setting("lang") {
        let lang_lower = saved_lang.to_lowercase();
        {
            let mut current_lang = CURRENT_LANGUAGE.write().unwrap();
            *current_lang = lang_lower;
        }
        let new_translations = load_translations()?;
        let mut translations = TRANSLATIONS.write().unwrap();
        *translations = new_translations;
    }
    Ok(())
}

pub fn set_language(lang: &str) -> Result<()> {
    let lang_lower = lang.to_lowercase();

    let lang_exists = AVAILABLE_LANGUAGES
        .iter()
        .any(|&l| l.to_lowercase() == lang_lower);

    if !lang_exists {
        return Err(AppError::Validation(get_translation(
            "system.commands.language.invalid",
            &[&lang_lower.to_uppercase()],
        )));
    }

    {
        let mut current_lang = CURRENT_LANGUAGE.write().unwrap();
        *current_lang = lang_lower.clone();
    }

    // Speichere die neue Spracheinstellung
    tokio::spawn(async move {
        if let Ok(mut config_handler) = ConfigHandler::new().await {
            if let Err(e) = config_handler
                .set_setting("lang".to_string(), lang_lower)
                .await
            {
                log::error!("Fehler beim Speichern der Spracheinstellung: {}", e);
            }
        }
    });

    // Lade Übersetzungen neu
    let new_translations = load_translations()?;
    let mut translations = TRANSLATIONS.write().unwrap();
    *translations = new_translations;

    Ok(())
}

pub fn get_current_language() -> String {
    CURRENT_LANGUAGE.read().unwrap().to_uppercase()
}

pub fn get_available_languages() -> Vec<String> {
    // Konvertiere zu Großbuchstaben nur bei der Ausgabe
    AVAILABLE_LANGUAGES
        .iter()
        .map(|&s| s.to_uppercase())
        .collect()
}
