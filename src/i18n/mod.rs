// ## FILE: src/i18n/mod.rs - KOMPRIMIERTE VERSION
use crate::core::prelude::*;
use crate::ui::color::AppColor;
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
        params
            .iter()
            .enumerate()
            .fold(self.text.clone(), |mut text, (i, param)| {
                text = text.replace(&format!("{{{}}}", i), param);
                if text.contains("{}") {
                    text = text.replacen("{}", param, 1);
                }
                text
            })
    }

    fn color(&self) -> AppColor {
        AppColor::from_any(&self.category)
    }
}

struct I18nService {
    language: String,
    entries: HashMap<String, Entry>,
    fallback: HashMap<String, Entry>,
    cache: HashMap<String, (String, AppColor)>,
    display_map: HashMap<String, String>,
}

impl I18nService {
    fn new() -> Self {
        Self {
            language: DEFAULT_LANGUAGE.into(),
            entries: HashMap::new(),
            fallback: HashMap::new(),
            cache: HashMap::new(),
            display_map: HashMap::new(),
        }
    }

    fn load_language(&mut self, lang: &str) -> Result<()> {
        // Validate
        if !Self::available_languages()
            .iter()
            .any(|l| l.eq_ignore_ascii_case(lang))
        {
            return Err(AppError::Translation(TranslationError::InvalidLanguage(
                lang.into(),
            )));
        }

        // Load entries
        self.entries = Self::load_entries(lang)?;
        if lang != DEFAULT_LANGUAGE {
            self.fallback = Self::load_entries(DEFAULT_LANGUAGE).unwrap_or_default();
        }

        self.build_display_map();
        self.cache.clear();
        self.language = lang.into();
        Ok(())
    }

    fn build_display_map(&mut self) {
        self.display_map.clear();
        for entry in self.entries.values().chain(self.fallback.values()) {
            self.display_map
                .entry(entry.display.to_lowercase())
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

        Ok(raw
            .iter()
            .filter_map(|(key, value)| {
                key.strip_suffix(".text").map(|base_key| {
                    let display = raw
                        .get(&format!("{}.display_text", base_key))
                        .unwrap_or(&base_key.to_uppercase())
                        .clone();
                    let category = raw
                        .get(&format!("{}.category", base_key))
                        .unwrap_or(&"info".to_string())
                        .clone();

                    (
                        base_key.into(),
                        Entry {
                            text: value.clone(),
                            display,
                            category,
                        },
                    )
                })
            })
            .collect())
    }

    fn get_translation(&mut self, key: &str, params: &[&str]) -> (String, AppColor) {
        // Cache key
        let cache_key = if params.is_empty() {
            key.into()
        } else {
            format!("{}:{}", key, params.join(":"))
        };

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // Get entry
        let (text, color) = match self.entries.get(key).or_else(|| self.fallback.get(key)) {
            Some(entry) => (entry.format(params), entry.color()),
            None => (format!("Missing: {}", key), AppColor::from_any("warning")),
        };

        // Cache with size limit
        if self.cache.len() >= 1000 {
            self.cache.clear();
        }
        self.cache.insert(cache_key, (text.clone(), color));
        (text, color)
    }

    fn get_command_translation(&mut self, key: &str, params: &[&str]) -> String {
        match self.entries.get(key).or_else(|| self.fallback.get(key)) {
            Some(entry) => format!("[{}] {}", entry.display, entry.format(params)),
            None => format!("[WARNING] Missing: {}", key),
        }
    }

    fn get_category_for_display(&self, display: &str) -> String {
        self.display_map
            .get(&display.to_lowercase())
            .cloned()
            .unwrap_or_else(|| "info".into())
    }

    fn available_languages() -> Vec<String> {
        Langs::iter()
            .filter_map(|f| {
                let filename = f.as_ref();
                filename.strip_suffix(".json").map(|s| s.to_uppercase())
            })
            .collect()
    }
}

// ✅ KOMPRIMIERTE SINGLETON
static SERVICE: std::sync::LazyLock<Arc<RwLock<I18nService>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(I18nService::new())));

// ✅ KOMPRIMIERTE PUBLIC API
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

pub fn get_color_category_for_display(display: &str) -> String {
    SERVICE.read().unwrap().get_category_for_display(display)
}

pub fn get_current_language() -> String {
    SERVICE.read().unwrap().language.to_uppercase()
}

pub fn get_available_languages() -> Vec<String> {
    I18nService::available_languages()
}

pub fn has_translation(key: &str) -> bool {
    let service = SERVICE.read().unwrap();
    service.entries.contains_key(key) || service.fallback.contains_key(key)
}

pub fn clear_translation_cache() {
    SERVICE.write().unwrap().cache.clear();
}

// ✅ KOMPRIMIERTE STATS & DEBUG
pub fn get_all_translation_keys() -> Vec<String> {
    let service = SERVICE.read().unwrap();
    let mut keys: Vec<String> = service
        .entries
        .keys()
        .chain(service.fallback.keys())
        .cloned()
        .collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

pub fn get_translation_stats() -> HashMap<String, usize> {
    let service = SERVICE.read().unwrap();
    [
        ("total_keys", service.entries.len()),
        ("fallback_keys", service.fallback.len()),
        ("cached_entries", service.cache.len()),
        ("display_mappings", service.display_map.len()),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}

pub fn get_missing_keys_report() -> Vec<String> {
    Vec::new()
}

// ✅ MACROS (unverändert)
#[macro_export]
macro_rules! t {
    ($key:expr) => { $crate::i18n::get_translation($key, &[]) };
    ($key:expr, $($arg:expr),+) => { $crate::i18n::get_translation($key, &[$($arg),+]) };
}

#[macro_export]
macro_rules! tc {
    ($key:expr) => { $crate::i18n::get_command_translation($key, &[]) };
    ($key:expr, $($arg:expr),+) => { $crate::i18n::get_command_translation($key, &[$($arg),+]) };
}
