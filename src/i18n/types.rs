// src/i18n/types.rs
use crate::core::error::{AppError, Result};
use crate::i18n::error::TranslationError;
use crate::ui::color::{AppColor, ColorCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationEntry {
    pub text: String,
    pub category: String,
}

impl TranslationEntry {
    pub fn get_color(&self) -> AppColor {
        AppColor::from_category(ColorCategory::from_str_or_default(&self.category))
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TranslationConfig {
    #[serde(skip)]
    entries: HashMap<String, TranslationEntry>,
}

impl TranslationConfig {
    pub fn load(lang: &str) -> Result<Self> {
        let translation_str = crate::i18n::langs::get_language_file(lang).ok_or_else(|| {
            AppError::Translation(TranslationError::LoadError(format!(
                "Sprachdatei für '{}' nicht gefunden",
                lang
            )))
        })?;

        let raw_entries: HashMap<String, String> =
            serde_json::from_str(translation_str).map_err(|e| {
                AppError::Translation(TranslationError::LoadError(format!(
                    "Fehler beim Parsen der Sprachdatei: {}",
                    e
                )))
            })?;

        // Konvertiere von flachen Schlüsseln zu TranslationEntries
        let mut entries = HashMap::new();

        for (key, value) in raw_entries.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5];
                let category_key = format!("{}.category", base_key);

                if let Some(category) = raw_entries.get(&category_key) {
                    entries.insert(
                        base_key.to_string(),
                        TranslationEntry {
                            text: value.clone(),
                            category: category.clone(),
                        },
                    );
                }
            }
        }

        Ok(Self { entries })
    }

    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        self.entries.get(key)
    }
}
