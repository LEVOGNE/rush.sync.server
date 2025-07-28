use crate::core::prelude::*;
use crate::ui::color::AppColor;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

pub const DEFAULT_LANGUAGE: &str = "en";

#[derive(Debug)]
pub enum TranslationError {
    InvalidLanguage(String),
    LoadError(String),
    ConfigError(String),
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLanguage(lang) => write!(f, "Ungültige Sprache: {}", lang),
            Self::LoadError(msg) => write!(f, "Ladefehler: {}", msg),
            Self::ConfigError(msg) => write!(f, "Konfigurationsfehler: {}", msg),
        }
    }
}

#[derive(RustEmbed)]
#[folder = "src/i18n/langs/"]
pub struct Langs;

fn get_language_file(lang: &str) -> Option<String> {
    let filename = format!("{}.json", lang.to_lowercase());
    Langs::get(&filename).and_then(|file| {
        std::str::from_utf8(file.data.as_ref())
            .ok()
            .map(|s| s.to_owned())
    })
}

#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub text: String,
    pub color_category: String,
    pub display_category: String,
}

impl TranslationEntry {
    pub fn get_color(&self) -> AppColor {
        AppColor::from_any(&self.color_category)
    }

    pub fn format(&self, params: &[&str]) -> (String, AppColor) {
        let mut text = self.text.clone();
        for param in params {
            text = text.replacen("{}", param, 1);
        }
        (text, self.get_color())
    }

    pub fn format_for_command(&self, params: &[&str]) -> String {
        let (text, _) = self.format(params);
        format!("[{}] {}", self.display_category.to_uppercase(), text)
    }
}

#[derive(Debug, Clone, Default)]
struct TranslationConfig {
    entries: HashMap<String, TranslationEntry>,
    // ✅ GLOBALES Display-Mapping - wird ERWEITERT statt ersetzt
    global_display_to_color_map: HashMap<String, String>,
}

impl TranslationConfig {
    fn load(lang: &str) -> Result<Self> {
        let translation_str = get_language_file(lang).ok_or_else(|| {
            AppError::Translation(TranslationError::LoadError(format!(
                "Language file for '{}' not found",
                lang
            )))
        })?;

        let raw_entries: HashMap<String, String> =
            serde_json::from_str(&translation_str).map_err(|e| {
                AppError::Translation(TranslationError::LoadError(format!(
                    "Error parsing language file: {}",
                    e
                )))
            })?;

        let mut entries = HashMap::new();
        let mut new_display_mappings = HashMap::new();

        for (key, value) in raw_entries.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5];
                let color_category = raw_entries
                    .get(&format!("{}.category", base_key))
                    .cloned()
                    .unwrap_or_else(|| "info".to_string());
                let display_category = raw_entries
                    .get(&format!("{}.display_category", base_key))
                    .cloned()
                    .unwrap_or_else(|| color_category.clone());

                entries.insert(
                    base_key.to_string(),
                    TranslationEntry {
                        text: value.clone(),
                        color_category: color_category.clone(),
                        display_category: display_category.clone(),
                    },
                );

                new_display_mappings.insert(
                    display_category.to_lowercase(),
                    color_category.to_lowercase(),
                );
            }
        }

        Ok(Self {
            entries,
            global_display_to_color_map: new_display_mappings,
        })
    }

    fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        self.entries.get(key)
    }

    fn get_color_category_for_display(&self, display_category: &str) -> String {
        self.global_display_to_color_map
            .get(&display_category.to_lowercase())
            .cloned()
            .unwrap_or_else(|| {
                // ✅ SMART FALLBACK: Versuche ähnliche Display-Categories zu finden
                self.find_similar_color_category(display_category)
            })
    }

    // ✅ SMART FALLBACK: Finde ähnliche Color-Categories
    fn find_similar_color_category(&self, display_category: &str) -> String {
        let display_lower = display_category.to_lowercase();

        // Versuche bekannte Patterns
        if display_lower.contains("error") || display_lower.contains("fehler") {
            return "error".to_string();
        }
        if display_lower.contains("warn") || display_lower.contains("warnung") {
            return "warning".to_string();
        }
        if display_lower.contains("info") {
            return "info".to_string();
        }
        if display_lower.contains("debug") {
            return "debug".to_string();
        }
        if display_lower.contains("lang")
            || display_lower.contains("sprache")
            || display_lower.contains("language")
        {
            return "lang".to_string();
        }
        if display_lower.contains("version") {
            return "version".to_string();
        }

        // Fallback
        "info".to_string()
    }

    // ✅ MERGE neue Display-Mappings mit bestehenden
    fn merge_display_mappings(&mut self, new_mappings: HashMap<String, String>) {
        for (display, color) in new_mappings {
            self.global_display_to_color_map.insert(display, color);
        }
    }
}

struct TranslationService {
    current_language: String,
    config: TranslationConfig,
    cache: Mutex<HashMap<String, (String, AppColor)>>,
}

impl TranslationService {
    fn new() -> Self {
        Self {
            current_language: DEFAULT_LANGUAGE.to_string(),
            config: TranslationConfig::default(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn get_translation_readonly(&self, key: &str, params: &[&str]) -> (String, AppColor) {
        let cache_key = if params.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", key, params.join(":"))
        };

        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        let (text, color) = if let Some(entry) = self.config.get_entry(key) {
            entry.format(params)
        } else {
            (
                format!("⚠️ Translation key not found: {}", key),
                AppColor::from_any("warning"),
            )
        };

        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() >= 1000 {
                cache.clear();
            }
            cache.insert(cache_key, (text.clone(), color));
        }

        (text, color)
    }

    fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    // ✅ NEUE METHODE: Merge Display-Mappings statt ersetzen
    fn update_language(&mut self, new_config: TranslationConfig) {
        // ✅ MERGE alte + neue Display-Mappings
        self.config
            .merge_display_mappings(new_config.global_display_to_color_map.clone());

        // Update entries
        self.config.entries = new_config.entries;

        // NUR Text-Cache leeren, Display-Mapping bleibt
        self.clear_cache();
    }
}

lazy_static! {
    static ref INSTANCE: RwLock<TranslationService> = RwLock::new(TranslationService::new());
}

pub async fn init() -> Result<()> {
    set_language_internal(DEFAULT_LANGUAGE, false)
}

pub fn set_language(lang: &str) -> Result<()> {
    set_language_internal(lang, true)
}

fn set_language_internal(lang: &str, _save_config: bool) -> Result<()> {
    let lang = lang.to_lowercase();

    if !get_available_languages()
        .iter()
        .any(|l| l.to_lowercase() == lang)
    {
        return Err(AppError::Translation(TranslationError::InvalidLanguage(
            lang.to_uppercase(),
        )));
    }

    let config = TranslationConfig::load(&lang).unwrap_or_default();
    let mut service = INSTANCE.write().unwrap();
    service.current_language = lang;

    // ✅ SMART UPDATE: Merge statt Replace
    service.update_language(config);

    Ok(())
}

pub fn get_translation(key: &str, params: &[&str]) -> String {
    INSTANCE
        .read()
        .unwrap()
        .get_translation_readonly(key, params)
        .0
}

pub fn get_command_translation(key: &str, params: &[&str]) -> String {
    let service = INSTANCE.read().unwrap();
    if let Some(entry) = service.config.get_entry(key) {
        entry.format_for_command(params)
    } else {
        format!("[WARNING] ⚠️ Translation key not found: {}", key)
    }
}

pub fn get_current_language() -> String {
    INSTANCE.read().unwrap().current_language.to_uppercase()
}

pub fn get_available_languages() -> Vec<String> {
    Langs::iter()
        .filter_map(|f| f.as_ref().strip_suffix(".json").map(|s| s.to_uppercase()))
        .collect()
}

pub fn get_color_category_for_display(display_category: &str) -> String {
    INSTANCE
        .read()
        .unwrap()
        .config
        .get_color_category_for_display(display_category)
}

pub fn clear_translation_cache() {
    INSTANCE.read().unwrap().clear_cache();
}
