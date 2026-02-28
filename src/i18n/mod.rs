use crate::core::prelude::*;
use crate::ui::color::AppColor;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::RwLock;

pub const DEFAULT_LANGUAGE: &str = "en";

#[derive(Debug)]
pub enum TranslationError {
    InvalidLanguage(String),
    LoadError(String),
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLanguage(lang) => write!(f, "Invalid language: {}", lang),
            Self::LoadError(msg) => write!(f, "Load error: {}", msg),
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
}

struct I18nService {
    language: String,
    entries: HashMap<String, Entry>,
    fallback: HashMap<String, Entry>,
    cache: RwLock<HashMap<String, String>>,
}

impl I18nService {
    fn new() -> Self {
        Self {
            language: DEFAULT_LANGUAGE.into(),
            entries: HashMap::new(),
            fallback: HashMap::new(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn load_language(&mut self, lang: &str) -> Result<()> {
        if !Self::available_languages()
            .iter()
            .any(|l| l.eq_ignore_ascii_case(lang))
        {
            return Err(AppError::Translation(TranslationError::InvalidLanguage(
                lang.into(),
            )));
        }

        self.entries = Self::load_entries(lang)?;

        // Load fallback from other languages
        self.fallback.clear();
        for available_lang in Self::available_languages() {
            if available_lang.to_lowercase() != lang.to_lowercase() {
                if let Ok(other_entries) = Self::load_entries(&available_lang.to_lowercase()) {
                    for (key, entry) in other_entries {
                        self.fallback.entry(key).or_insert(entry);
                    }
                }
            }
        }

        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
        self.language = lang.into();
        Ok(())
    }

    fn load_entries(lang: &str) -> Result<HashMap<String, Entry>> {
        let lang_lower = lang.to_lowercase();
        let mut merged_raw: HashMap<String, String> = HashMap::new();

        let category_files: Vec<String> = Langs::iter()
            .filter_map(|file| {
                let filename = file.as_ref();
                let prefix = format!("{}/", lang_lower);

                if filename.starts_with(&prefix) && filename.ends_with(".json") {
                    Some(filename.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut found_modular = false;
        for filename in category_files {
            if let Some(content) = Langs::get(&filename) {
                if let Ok(content_str) = std::str::from_utf8(content.data.as_ref()) {
                    if let Ok(raw) = serde_json::from_str::<HashMap<String, String>>(content_str) {
                        merged_raw.extend(raw);
                        found_modular = true;
                    }
                }
            }
        }

        // Fallback: single-file format
        if !found_modular {
            let filename = format!("{}.json", lang_lower);
            let content = Langs::get(&filename).ok_or_else(|| {
                AppError::Translation(TranslationError::LoadError(format!(
                    "File not found: {}",
                    filename
                )))
            })?;

            let content_str = std::str::from_utf8(content.data.as_ref())
                .map_err(|e| AppError::Translation(TranslationError::LoadError(e.to_string())))?;

            merged_raw = serde_json::from_str(content_str)
                .map_err(|e| AppError::Translation(TranslationError::LoadError(e.to_string())))?;
        }

        Ok(merged_raw
            .iter()
            .filter_map(|(key, value)| {
                key.strip_suffix(".text").map(|base_key| {
                    let display = merged_raw
                        .get(&format!("{}.display_text", base_key))
                        .unwrap_or(&base_key.to_uppercase())
                        .clone();
                    let category = merged_raw
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

    // Now takes &self - cache has its own lock
    fn get_translation(&self, key: &str, params: &[&str]) -> String {
        let cache_key = if params.is_empty() {
            key.into()
        } else {
            format!("{}:{}", key, params.join(":"))
        };

        // Fast path: read lock on cache
        if let Ok(cache) = self.cache.read() {
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        // Slow path: compute and write to cache
        let text = match self.entries.get(key).or_else(|| self.fallback.get(key)) {
            Some(entry) => entry.format(params),
            None => format!("Missing: {}", key),
        };

        if let Ok(mut cache) = self.cache.write() {
            if cache.len() >= 1000 {
                cache.clear();
            }
            cache.insert(cache_key, text.clone());
        }
        text
    }

    fn get_command_translation(&self, key: &str, params: &[&str]) -> String {
        match self.entries.get(key).or_else(|| self.fallback.get(key)) {
            Some(entry) => format!("[{}] {}", entry.display, entry.format(params)),
            None => format!("[WARNING] Missing: {}", key),
        }
    }

    fn get_display_color(&self, display_text: &str) -> AppColor {
        for entry in self.entries.values() {
            if entry.display.to_uppercase() == display_text.to_uppercase() {
                return AppColor::from_category(&entry.category);
            }
        }
        AppColor::from_any("info")
    }

    fn available_languages() -> Vec<String> {
        let mut languages = std::collections::HashSet::new();

        for file in Langs::iter() {
            let filename = file.as_ref();

            if filename.ends_with(".json") {
                if let Some(lang) = filename.strip_suffix(".json") {
                    if !lang.contains('/') {
                        languages.insert(lang.to_uppercase());
                    }
                }

                if let Some(slash_pos) = filename.find('/') {
                    let lang = &filename[..slash_pos];
                    languages.insert(lang.to_uppercase());
                }
            }
        }

        languages.into_iter().collect()
    }
}

static SERVICE: std::sync::LazyLock<RwLock<I18nService>> =
    std::sync::LazyLock::new(|| RwLock::new(I18nService::new()));

pub async fn init() -> Result<()> {
    set_language(DEFAULT_LANGUAGE)
}

pub fn set_language(lang: &str) -> Result<()> {
    match SERVICE.write() {
        Ok(mut service) => service.load_language(lang),
        Err(e) => Err(AppError::Validation(format!("i18n lock poisoned: {}", e))),
    }
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    match SERVICE.read() {
        Ok(service) => service.get_translation(key, params),
        Err(_) => format!("Missing: {}", key),
    }
}

pub fn get_command_translation(key: &str, params: &[&str]) -> String {
    match SERVICE.read() {
        Ok(service) => service.get_command_translation(key, params),
        Err(_) => format!("[WARNING] Missing: {}", key),
    }
}

pub fn get_color_for_display_text(display_text: &str) -> AppColor {
    match SERVICE.read() {
        Ok(service) => service.get_display_color(display_text),
        Err(_) => AppColor::from_any("info"),
    }
}

pub fn get_color_category_for_display(display: &str) -> String {
    match display.to_lowercase().as_str() {
        "theme" => "theme".to_string(),
        "lang" | "sprache" => "lang".to_string(),
        _ => "info".to_string(),
    }
}

pub fn get_current_language() -> String {
    match SERVICE.read() {
        Ok(service) => service.language.to_uppercase(),
        Err(_) => DEFAULT_LANGUAGE.to_uppercase(),
    }
}

pub fn get_available_languages() -> Vec<String> {
    I18nService::available_languages()
}

pub fn has_translation(key: &str) -> bool {
    match SERVICE.read() {
        Ok(service) => service.entries.contains_key(key) || service.fallback.contains_key(key),
        Err(_) => false,
    }
}

pub fn clear_translation_cache() {
    if let Ok(service) = SERVICE.read() {
        if let Ok(mut cache) = service.cache.write() {
            cache.clear();
        }
    }
}

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
