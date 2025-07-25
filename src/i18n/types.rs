// i18n/types.rs - MIT DEBUG OUTPUT
use crate::core::error::{AppError, Result};
use crate::i18n::error::TranslationError;
use crate::ui::color::AppColor;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub text: String,
    pub category: String,
}

impl TranslationEntry {
    pub fn get_color(&self) -> AppColor {
        AppColor::from_category_str(&self.category)
    }

    pub fn format(&self, params: &[&str]) -> (String, AppColor) {
        let text = if params.is_empty() {
            self.text.clone()
        } else {
            let mut result = self.text.clone();
            for param in params.iter() {
                result = result.replacen("{}", param, 1);
            }
            result
        };

        (text, self.get_color())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TranslationConfig {
    entries: HashMap<String, TranslationEntry>,
}

impl TranslationConfig {
    pub fn load(lang: &str) -> Result<Self> {
        log::debug!("🔍 Loading translation for language: {}", lang);

        let translation_str = crate::i18n::langs::get_language_file(lang).ok_or_else(|| {
            log::error!("❌ Language file not found for: {}", lang);
            AppError::Translation(TranslationError::LoadError(format!(
                "Sprachdatei für '{}' nicht gefunden",
                lang
            )))
        })?;

        log::debug!("📄 JSON content length: {} chars", translation_str.len());

        // ✅ PARSE FLACHE JSON-STRUKTUR
        let raw_entries: HashMap<String, String> =
            serde_json::from_str(translation_str).map_err(|e| {
                log::error!("❌ JSON parse error: {}", e);
                AppError::Translation(TranslationError::LoadError(format!(
                    "Fehler beim Parsen der Sprachdatei: {}",
                    e
                )))
            })?;

        log::debug!("📋 Raw entries loaded: {}", raw_entries.len());

        // ✅ KONVERTIERE .text/.category PAARE
        let mut entries = HashMap::new();
        let mut processed_count = 0;
        let mut skipped_count = 0;

        for (key, value) in raw_entries.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5]; // Remove ".text"
                let category_key = format!("{}.category", base_key);

                if let Some(category) = raw_entries.get(&category_key) {
                    log::debug!(
                        "✅ Processing: {} -> text: '{}', category: '{}'",
                        base_key,
                        value.chars().take(30).collect::<String>(),
                        category
                    );

                    entries.insert(
                        base_key.to_string(),
                        TranslationEntry {
                            text: value.clone(),
                            category: category.clone(),
                        },
                    );
                    processed_count += 1;
                } else {
                    log::warn!(
                        "⚠️  Missing category for key: {} (expected: {})",
                        key,
                        category_key
                    );
                    skipped_count += 1;
                }
            }
        }

        log::info!(
            "🎯 Translation processing complete: {} entries processed, {} skipped",
            processed_count,
            skipped_count
        );

        // Debug: Liste alle verarbeiteten Keys
        log::debug!("📝 Processed translation keys:");
        for key in entries.keys() {
            log::debug!("  • {}", key);
        }

        Ok(Self { entries })
    }

    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        let result = self.entries.get(key);
        if result.is_none() {
            log::warn!("🔍 Translation key not found in config: '{}'", key);
            log::debug!(
                "Available keys: {:?}",
                self.entries.keys().collect::<Vec<_>>()
            );
        }
        result
    }
}
