// src/i18n/mod.rs
use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

// Konstanten
mod langs;
use langs::{AVAILABLE_LANGUAGES, DEFAULT_LANGUAGE};
const MAX_CACHE_SIZE: usize = 1000; // Begrenzt Cache-Größe

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationConfig {
    system: SystemTranslations,
    #[serde(skip)]
    cache: TranslationCache,
}

#[derive(Debug, Clone, Default)]
struct TranslationCache {
    entries: HashMap<String, String>,
    hits: usize,
    misses: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct LogTranslations {
    info: String,
    error: String,
    warn: String,
    debug: String,
    trace: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct SystemTranslations {
    startup: StartupTranslations,
    commands: CommandTranslations,
    log: LogTranslations, // Neues Feld für Log-Übersetzungen
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct StartupTranslations {
    version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct CommandTranslations {
    unknown: String,
    language: LanguageTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct LanguageTranslations {
    current: String,
    changed: String,
    invalid: String,
    available: String,
}

// Standardisierte Fehlertypen für Übersetzungen
#[derive(Debug)]
pub enum TranslationError {
    InvalidLanguage(String),
    LoadError(String),
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLanguage(lang) => write!(
                f,
                "{}",
                get_translation("system.commands.language.invalid", &[lang])
            ),
            Self::LoadError(msg) => write!(f, "{}", msg),
        }
    }
}

lazy_static! {
    static ref TRANSLATION_STATE: RwLock<TranslationState> = RwLock::new(TranslationState::new());
}

#[derive(Debug)]
struct TranslationState {
    current_language: String,
    translations: TranslationConfig,
}

impl TranslationState {
    fn new() -> Self {
        let translations = load_translations(DEFAULT_LANGUAGE)
            .unwrap_or_else(|e| panic!("Kritischer Fehler beim Laden der Standardsprache: {}", e));

        Self {
            current_language: DEFAULT_LANGUAGE.to_string(),
            translations,
        }
    }

    fn update_language(&mut self, new_lang: &str) -> Result<()> {
        let translations = load_translations(new_lang)?;
        self.current_language = new_lang.to_string();
        self.translations = translations;
        Ok(())
    }
}

impl TranslationCache {
    fn get(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.entries.get(key) {
            self.hits += 1;
            Some(value.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    fn insert(&mut self, key: String, value: String) {
        if self.entries.len() >= MAX_CACHE_SIZE {
            self.entries.clear();
            log::debug!("Translation cache cleared due to size limit");
        }
        self.entries.insert(key, value);
    }

    fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            system: SystemTranslations::default(),
            cache: TranslationCache::default(),
        }
    }
}

impl TranslationConfig {
    fn get_template(&self, key: &str) -> Option<&String> {
        let parts: Vec<&str> = key.split('.').collect();
        match parts.as_slice() {
            ["system", "startup", "version"] => Some(&self.system.startup.version),
            ["system", "commands", "unknown"] => Some(&self.system.commands.unknown),
            ["system", "commands", "language", "current"] => {
                Some(&self.system.commands.language.current)
            }
            ["system", "commands", "language", "changed"] => {
                Some(&self.system.commands.language.changed)
            }
            ["system", "commands", "language", "invalid"] => {
                Some(&self.system.commands.language.invalid)
            }
            ["system", "commands", "language", "available"] => {
                Some(&self.system.commands.language.available)
            }
            ["system", "log", level] => {
                // Einfacher String-Vergleich
                if level == &"info" {
                    Some(&self.system.log.info)
                } else if level == &"error" {
                    Some(&self.system.log.error)
                } else if level == &"warn" {
                    Some(&self.system.log.warn)
                } else if level == &"debug" {
                    Some(&self.system.log.debug)
                } else if level == &"trace" {
                    Some(&self.system.log.trace)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn load_translations(lang: &str) -> Result<TranslationConfig> {
    let translation_str = langs::get_language_file(lang).unwrap_or_else(|| {
        langs::get_language_file(DEFAULT_LANGUAGE).expect("Default language file not found")
    });

    serde_json::from_str::<TranslationConfig>(translation_str)
        .map_err(|e| AppError::Validation(format!("Übersetzungsfehler: {}", e)))
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    let mut state = TRANSLATION_STATE.write().unwrap();

    let cache_key = if params.is_empty() {
        key.to_string()
    } else {
        format!("{}:{}", key, params.join(":"))
    };

    if let Some(cached) = state.translations.cache.get(&cache_key) {
        return cached;
    }

    let translated = match state.translations.get_template(key) {
        Some(template) => {
            if let Some(param) = params.first() {
                template.replace("{}", param)
            } else {
                template.clone()
            }
        }
        None => format!("Translation key not found: {}", key),
    };

    state
        .translations
        .cache
        .insert(cache_key, translated.clone());
    translated
}

pub async fn init_language_silent() -> Result<()> {
    if let Ok(config_handler) = ConfigHandler::new().await {
        if let Some(saved_lang) = config_handler.get_setting("lang") {
            return set_language_internal(&saved_lang.to_lowercase(), false);
        }
    }
    Ok(())
}

fn set_language_internal(lang: &str, save_config: bool) -> Result<()> {
    let lang_lower = lang.to_lowercase();

    if !AVAILABLE_LANGUAGES.iter().any(|&l| l == lang_lower) {
        return Err(AppError::Translation(TranslationError::InvalidLanguage(
            lang_lower.to_uppercase(),
        )));
    }

    // Sprache aktualisieren
    TRANSLATION_STATE
        .write()
        .unwrap()
        .update_language(&lang_lower)?;

    // Optional: Konfiguration speichern
    if save_config {
        tokio::spawn(async move {
            if let Ok(mut config_handler) = ConfigHandler::new().await {
                if let Err(e) = config_handler
                    .set_setting("lang".to_string(), lang_lower)
                    .await
                {
                    // Log-Nachricht auch übersetzen
                    log::error!(
                        "{}",
                        get_translation("system.error.config_save", &[&e.to_string()])
                    );
                }
            }
        });
    }

    Ok(()) // Hier fehlten die beiden Klammern
}

pub fn set_language(lang: &str) -> Result<()> {
    set_language_internal(lang, true)
}

pub fn get_current_language() -> String {
    TRANSLATION_STATE
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
    TRANSLATION_STATE.read().unwrap().translations.cache.stats()
}
