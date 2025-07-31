use crate::core::prelude::*;
use crate::ui::color::AppColor;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const DEFAULT_LANGUAGE: &str = "en";

#[derive(Debug)]
pub enum TranslationError {
    InvalidLanguage(String),
    LoadError(String),
    KeyNotFound(String),
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLanguage(lang) => write!(f, "Invalid language: {}", lang),
            Self::LoadError(msg) => write!(f, "Load error: {}", msg),
            Self::KeyNotFound(key) => write!(f, "Translation key not found: {}", key),
        }
    }
}

#[derive(RustEmbed)]
#[folder = "src/i18n/langs/"]
pub struct Langs;

#[derive(Debug, Clone)]
struct Entry {
    text: String,
    display: String,
    category: String,
}

impl Entry {
    fn format(&self, params: &[&str]) -> String {
        let mut text = self.text.clone();
        for (i, param) in params.iter().enumerate() {
            text = text.replace(&format!("{{{}}}", i), param);
        }
        for param in params {
            if text.contains("{}") {
                text = text.replacen("{}", param, 1);
            }
        }
        text
    }

    fn get_color(&self) -> AppColor {
        AppColor::from_any(&self.category)
    }
}

struct I18nService {
    language: String,
    entries: HashMap<String, Entry>,
    fallback: HashMap<String, Entry>,
    cache: HashMap<String, (String, AppColor)>,
    display_to_category: HashMap<String, String>,
}

impl I18nService {
    fn new() -> Self {
        Self {
            language: DEFAULT_LANGUAGE.to_string(),
            entries: HashMap::new(),
            fallback: HashMap::new(),
            cache: HashMap::new(),
            display_to_category: HashMap::new(),
        }
    }

    fn load_language(&mut self, lang: &str) -> Result<()> {
        if !Self::get_available_languages()
            .iter()
            .any(|l| l.to_lowercase() == lang.to_lowercase())
        {
            return Err(AppError::Translation(TranslationError::InvalidLanguage(
                lang.to_string(),
            )));
        }

        self.entries = Self::load_entries(lang)?;

        if lang != DEFAULT_LANGUAGE {
            self.fallback = Self::load_entries(DEFAULT_LANGUAGE).unwrap_or_default();
        }

        self.build_display_to_category_mapping();
        self.cache.clear();
        self.language = lang.to_string();

        log::info!(
            "Language loaded: {} ({} keys)",
            lang.to_uppercase(),
            self.entries.len()
        );
        Ok(())
    }

    fn build_display_to_category_mapping(&mut self) {
        self.display_to_category.clear();

        for entry in self.entries.values() {
            let display_key = entry.display.to_lowercase();
            self.display_to_category
                .insert(display_key, entry.category.clone());
        }

        for entry in self.fallback.values() {
            let display_key = entry.display.to_lowercase();
            self.display_to_category
                .entry(display_key)
                .or_insert_with(|| entry.category.clone());
        }
    }

    fn load_entries(lang: &str) -> Result<HashMap<String, Entry>> {
        let filename = format!("{}.json", lang.to_lowercase());
        let content = Langs::get(&filename).ok_or_else(|| {
            AppError::Translation(TranslationError::LoadError(format!(
                "File not found: {}",
                filename
            )))
        })?;

        let content_str = std::str::from_utf8(content.data.as_ref())
            .map_err(|e| AppError::Translation(TranslationError::LoadError(e.to_string())))?;

        let raw: HashMap<String, String> = serde_json::from_str(content_str)
            .map_err(|e| AppError::Translation(TranslationError::LoadError(e.to_string())))?;

        let mut entries = HashMap::new();

        for (key, value) in raw.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5];
                let display = raw
                    .get(&format!("{}.display_text", base_key))
                    .unwrap_or(&base_key.to_uppercase())
                    .clone();
                let category = raw
                    .get(&format!("{}.category", base_key))
                    .unwrap_or(&"info".to_string())
                    .clone();

                entries.insert(
                    base_key.to_string(),
                    Entry {
                        text: value.clone(),
                        display,
                        category,
                    },
                );
            }
        }

        Ok(entries)
    }

    fn get_translation(&mut self, key: &str, params: &[&str]) -> (String, AppColor) {
        let cache_key = if params.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", key, params.join(":"))
        };

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let entry = self.entries.get(key).or_else(|| self.fallback.get(key));

        let (text, color) = match entry {
            Some(e) => (e.format(params), e.get_color()),
            None => {
                log::warn!("Missing key: {}", key);
                (format!("Missing: {}", key), AppColor::from_any("warning"))
            }
        };

        if self.cache.len() >= 1000 {
            self.cache.clear();
        }
        self.cache.insert(cache_key, (text.clone(), color));

        (text, color)
    }

    fn get_command_translation(&mut self, key: &str, params: &[&str]) -> String {
        if let Some(entry) = self.entries.get(key).or_else(|| self.fallback.get(key)) {
            format!("[{}] {}", entry.display, entry.format(params))
        } else {
            format!("[WARNING] Missing: {}", key)
        }
    }

    fn get_category_for_display(&self, display_text: &str) -> String {
        let display_lower = display_text.to_lowercase();

        if let Some(category) = self.display_to_category.get(&display_lower) {
            return category.clone();
        }

        "info".to_string()
    }

    fn get_available_languages() -> Vec<String> {
        Langs::iter()
            .filter_map(|f| f.as_ref().strip_suffix(".json").map(|s| s.to_uppercase()))
            .collect()
    }

    fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

lazy_static! {
    static ref SERVICE: Arc<RwLock<I18nService>> = Arc::new(RwLock::new(I18nService::new()));
}

// Öffentliche API
pub async fn init() -> Result<()> {
    set_language(DEFAULT_LANGUAGE)
}

pub fn set_language(lang: &str) -> Result<()> {
    SERVICE.write().unwrap().load_language(lang)
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    SERVICE.write().unwrap().get_translation(key, params).0
}

pub fn get_command_translation(key: &str, params: &[&str]) -> String {
    SERVICE
        .write()
        .unwrap()
        .get_command_translation(key, params)
}

pub fn get_color_category_for_display(display_category: &str) -> String {
    SERVICE
        .read()
        .unwrap()
        .get_category_for_display(display_category)
}

pub fn get_current_language() -> String {
    SERVICE.read().unwrap().language.to_uppercase()
}

pub fn get_available_languages() -> Vec<String> {
    I18nService::get_available_languages()
}

pub fn has_translation(key: &str) -> bool {
    let service = SERVICE.read().unwrap();
    service.entries.contains_key(key) || service.fallback.contains_key(key)
}

pub fn clear_translation_cache() {
    SERVICE.write().unwrap().clear_cache();
}

pub fn get_all_translation_keys() -> Vec<String> {
    let service = SERVICE.read().unwrap();
    let mut keys: Vec<String> = service.entries.keys().cloned().collect();
    keys.extend(service.fallback.keys().cloned());
    keys.sort();
    keys.dedup();
    keys
}

pub fn get_translation_stats() -> HashMap<String, usize> {
    let service = SERVICE.read().unwrap();
    let mut stats = HashMap::new();
    stats.insert("total_keys".to_string(), service.entries.len());
    stats.insert("fallback_keys".to_string(), service.fallback.len());
    stats.insert("cached_entries".to_string(), service.cache.len());
    stats.insert(
        "display_mappings".to_string(),
        service.display_to_category.len(),
    );
    stats
}

pub fn get_missing_keys_report() -> Vec<String> {
    Vec::new()
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::get_translation($key, &[])
    };
    ($key:expr, $($arg:expr),+) => {
        $crate::i18n::get_translation($key, &[$($arg),+])
    };
}

#[macro_export]
macro_rules! tc {
    ($key:expr) => {
        $crate::i18n::get_command_translation($key, &[])
    };
    ($key:expr, $($arg:expr),+) => {
        $crate::i18n::get_command_translation($key, &[$($arg),+])
    };
}
