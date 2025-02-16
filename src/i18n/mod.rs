// src/i18n/mod.rs
use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use crate::ui::color::ColorCategory;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

mod langs;
use langs::{AVAILABLE_LANGUAGES, DEFAULT_LANGUAGE};
const MAX_CACHE_SIZE: usize = 1000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationEntry {
    pub text: String,
    pub category: String,
}

impl Default for TranslationEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            category: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageTranslations {
    pub current: TranslationEntry,
    pub changed: TranslationEntry,
    pub invalid: TranslationEntry,
    pub available: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandTranslations {
    pub unknown: TranslationEntry,
    pub language: LanguageTranslations,
    pub version: TranslationEntry,
}

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
struct SystemTranslations {
    startup: StartupTranslations,
    commands: CommandTranslations,
    log: LogTranslations,
    input: InputTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct InputTranslations {
    confirm_exit: TranslationEntry,
    cancelled: TranslationEntry,
    confirm: InputConfirmTranslations,
    cancel: InputConfirmTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct InputConfirmTranslations {
    short: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct StartupTranslations {
    version: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct LogTranslations {
    info: TranslationEntry,
    error: TranslationEntry,
    warn: TranslationEntry,
    debug: TranslationEntry,
    trace: TranslationEntry,
}

#[derive(Debug)]
pub enum TranslationError {
    InvalidLanguage(String),
    LoadError(String),
}

impl Default for LanguageTranslations {
    fn default() -> Self {
        Self {
            current: TranslationEntry::default(),
            changed: TranslationEntry::default(),
            invalid: TranslationEntry::default(),
            available: TranslationEntry::default(),
        }
    }
}

impl Default for CommandTranslations {
    fn default() -> Self {
        Self {
            unknown: TranslationEntry::default(),
            language: LanguageTranslations::default(),
            version: TranslationEntry::default(),
        }
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

    /*  fn update_language(&mut self, new_lang: &str) -> Result<()> {
        let translations = load_translations(new_lang)?;
        self.current_language = new_lang.to_string();
        self.translations = translations;
        Ok(())
    } */
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

impl TranslationConfig {
    fn get_template(&self, key: &str) -> Option<(String, ColorCategory)> {
        let parts: Vec<&str> = key.split('.').collect();
        match parts.as_slice() {
            // Startup
            ["system", "startup", "version"] => Some((
                self.system.startup.version.text.clone(),
                ColorCategory::from_str(&self.system.startup.version.category),
            )),

            // Commands
            ["system", "commands", "unknown"] => Some((
                self.system.commands.unknown.text.clone(),
                ColorCategory::from_str(&self.system.commands.unknown.category),
            )),
            ["system", "commands", "version"] => Some((
                self.system.commands.version.text.clone(),
                ColorCategory::from_str(&self.system.commands.version.category),
            )),
            ["system", "commands", "language", "current"] => Some((
                self.system.commands.language.current.text.clone(),
                ColorCategory::from_str(&self.system.commands.language.current.category),
            )),
            ["system", "commands", "language", "changed"] => Some((
                self.system.commands.language.changed.text.clone(),
                ColorCategory::from_str(&self.system.commands.language.changed.category),
            )),
            ["system", "commands", "language", "invalid"] => Some((
                self.system.commands.language.invalid.text.clone(),
                ColorCategory::from_str(&self.system.commands.language.invalid.category),
            )),
            ["system", "commands", "language", "available"] => Some((
                self.system.commands.language.available.text.clone(),
                ColorCategory::from_str(&self.system.commands.language.available.category),
            )),

            // Input
            ["system", "input", "confirm_exit"] => Some((
                self.system.input.confirm_exit.text.clone(),
                ColorCategory::from_str(&self.system.input.confirm_exit.category),
            )),
            ["system", "input", "cancelled"] => Some((
                self.system.input.cancelled.text.clone(),
                ColorCategory::from_str(&self.system.input.cancelled.category),
            )),
            ["system", "input", "confirm", "short"] => Some((
                self.system.input.confirm.short.text.clone(),
                ColorCategory::from_str(&self.system.input.confirm.short.category),
            )),
            ["system", "input", "cancel", "short"] => Some((
                self.system.input.cancel.short.text.clone(),
                ColorCategory::from_str(&self.system.input.cancel.short.category),
            )),

            // Logs
            ["system", "log", level] => match *level {
                "info" => Some((
                    self.system.log.info.text.clone(),
                    ColorCategory::from_str(&self.system.log.info.category),
                )),
                "error" => Some((
                    self.system.log.error.text.clone(),
                    ColorCategory::from_str(&self.system.log.error.category),
                )),
                "warn" => Some((
                    self.system.log.warn.text.clone(),
                    ColorCategory::from_str(&self.system.log.warn.category),
                )),
                "debug" => Some((
                    self.system.log.debug.text.clone(),
                    ColorCategory::from_str(&self.system.log.debug.category),
                )),
                "trace" => Some((
                    self.system.log.trace.text.clone(),
                    ColorCategory::from_str(&self.system.log.trace.category),
                )),
                _ => None,
            },
            _ => None,
        }
    }
}

fn load_translations(lang: &str) -> Result<TranslationConfig> {
    let translation_str = match langs::get_language_file(lang) {
        Some(content) => content,
        None => {
            langs::get_language_file(DEFAULT_LANGUAGE).expect("Default language file not found")
        }
    };

    match serde_json::from_str::<TranslationConfig>(translation_str) {
        Ok(config) => Ok(config),
        Err(e) => Err(AppError::Validation(format!("Übersetzungsfehler: {}", e))),
    }
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
        Some((template, _category)) => {
            if let Some(param) = params.first() {
                template.replace("{}", param)
            } else {
                template
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

pub fn get_translation_details(key: &str) -> (String, ColorCategory) {
    let state = TRANSLATION_STATE.read().unwrap();

    match state.translations.get_template(key) {
        Some((template, category)) => (template, category),
        None => (
            format!("Translation key not found: {}", key),
            ColorCategory::Default,
        ),
    }
}

pub async fn init_language_silent() -> Result<()> {
    match ConfigHandler::new().await {
        Ok(config_handler) => {
            if let Some(saved_lang) = config_handler.get_setting("lang") {
                return set_language_internal(&saved_lang.to_lowercase(), false);
            } else {
                return set_language_internal(DEFAULT_LANGUAGE, false);
            }
        }
        Err(_e) => {
            return set_language_internal(DEFAULT_LANGUAGE, false);
        }
    }
}

fn set_language_internal(lang: &str, save_config: bool) -> Result<()> {
    let lang_lower = lang.to_lowercase();

    if !AVAILABLE_LANGUAGES.iter().any(|&l| l == lang_lower) {
        return Err(AppError::Translation(TranslationError::InvalidLanguage(
            lang_lower.to_uppercase(),
        )));
    }

    // Lade die Übersetzungen zuerst
    let translations = match load_translations(&lang_lower) {
        Ok(trans) => trans,
        Err(e) => {
            log::error!("Fehler beim Laden der Übersetzungen: {:?}", e);
            return Err(e);
        }
    };

    // Dann aktualisiere den State
    match TRANSLATION_STATE.write() {
        Ok(mut state) => {
            state.current_language = lang_lower;
            state.translations = translations;
        }
        Err(e) => {
            return Err(AppError::Validation(format!(
                "Translation State Error: {:?}",
                e
            )));
        }
    }

    if save_config {
        log::debug!("Überspringe Konfigurationsspeicherung");
    }

    ("Sprachinitialisierung abgeschlossen");
    Ok(())
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
