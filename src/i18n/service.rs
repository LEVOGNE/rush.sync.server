// i18n/service.rs - VOLLSTÄNDIG
use super::DEFAULT_LANGUAGE;
use crate::i18n::cache::TranslationCache;
use crate::i18n::types::TranslationConfig;
use crate::ui::color::AppColor;
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    static ref INSTANCE: RwLock<TranslationService> = RwLock::new(TranslationService::new());
}

pub struct TranslationService {
    pub(crate) current_language: String,
    pub(crate) config: TranslationConfig,
    pub(crate) cache: TranslationCache,
}

impl TranslationService {
    pub fn new() -> Self {
        Self {
            current_language: DEFAULT_LANGUAGE.to_string(),
            config: TranslationConfig::default(),
            cache: TranslationCache::new(1000),
        }
    }

    pub fn get_instance() -> &'static RwLock<TranslationService> {
        &INSTANCE
    }

    // ✅ DIREKTE String + AppColor Rückgabe
    pub fn get_translation(&mut self, key: &str, params: &[&str]) -> (String, AppColor) {
        let cache_key = self.build_cache_key(key, params);

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached;
        }

        let (text, color) = if let Some(entry) = self.config.get_entry(key) {
            entry.format(params)
        } else {
            // Fallback mit Warning-Farbe
            (
                format!("⚠️ Translation key not found: {}", key),
                AppColor::from_category_str("warning"),
            )
        };

        self.cache.insert(cache_key, (text.clone(), color));
        (text, color)
    }

    fn build_cache_key(&self, key: &str, params: &[&str]) -> String {
        if params.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", key, params.join(":"))
        }
    }
}
