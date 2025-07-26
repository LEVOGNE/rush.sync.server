// i18n/types.rs - EINFACHE UND KORREKTE LÖSUNG
use crate::core::error::{AppError, Result};
use crate::i18n::error::TranslationError;
use crate::ui::color::AppColor;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub text: String,
    pub color_category: String,
    pub display_category: String,
}

impl TranslationEntry {
    pub fn get_color(&self) -> AppColor {
        AppColor::from_category_str(&self.color_category)
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

    pub fn format_for_command(&self, params: &[&str]) -> String {
        let (text, _color) = self.format(params);
        format!("[{}] {}", self.display_category.to_uppercase(), text)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TranslationConfig {
    entries: HashMap<String, TranslationEntry>,
    display_to_color_map: HashMap<String, String>,
}

impl TranslationConfig {
    pub fn load(lang: &str) -> Result<Self> {
        // ✅ KEINE DEBUG-MESSAGES MEHR - Still laden

        let translation_str = crate::i18n::langs::get_language_file(lang).ok_or_else(|| {
            log::error!("Language file not found: {}", lang);
            AppError::Translation(TranslationError::LoadError(format!(
                "Language file for '{}' not found", // ✅ GENERIC ENGLISH
                lang
            )))
        })?;

        let raw_entries: HashMap<String, String> =
            serde_json::from_str(translation_str).map_err(|e| {
                log::error!("JSON parse error: {}", e);
                AppError::Translation(TranslationError::LoadError(format!(
                    "Error parsing language file: {}", // ✅ GENERIC ENGLISH
                    e
                )))
            })?;

        let mut entries = HashMap::new();
        let mut display_to_color_map = HashMap::new();

        for (key, value) in raw_entries.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5];
                let (color_category, display_category) =
                    Self::determine_categories(&raw_entries, base_key);

                if color_category.is_none() {
                    // ✅ SIMPLE WARNING - beim Laden, kein i18n verfügbar
                    log::warn!("No category found for key: {}", base_key);
                    continue;
                }

                let color_cat = color_category.unwrap();
                let display_cat = display_category.unwrap_or_else(|| color_cat.clone());

                entries.insert(
                    base_key.to_string(),
                    TranslationEntry {
                        text: value.clone(),
                        color_category: color_cat.clone(),
                        display_category: display_cat.clone(),
                    },
                );

                display_to_color_map.insert(display_cat.to_lowercase(), color_cat.to_lowercase());
            }
        }

        // ✅ KEINE INFO-MESSAGE MEHR - Still laden

        Ok(Self {
            entries,
            display_to_color_map,
        })
    }

    fn determine_categories(
        raw_entries: &HashMap<String, String>,
        base_key: &str,
    ) -> (Option<String>, Option<String>) {
        let color_category_key = format!("{}.color_category", base_key);
        let display_category_key = format!("{}.display_category", base_key);

        if let (Some(color_cat), Some(display_cat)) = (
            raw_entries.get(&color_category_key),
            raw_entries.get(&display_category_key),
        ) {
            return (Some(color_cat.clone()), Some(display_cat.clone()));
        }

        let category_key = format!("{}.category", base_key);

        if let (Some(color_cat), Some(display_cat)) = (
            raw_entries.get(&category_key),
            raw_entries.get(&display_category_key),
        ) {
            return (Some(color_cat.clone()), Some(display_cat.clone()));
        }

        if let Some(legacy_cat) = raw_entries.get(&category_key) {
            return (Some(legacy_cat.clone()), Some(legacy_cat.clone()));
        }

        (None, None)
    }

    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        let result = self.entries.get(key);
        if result.is_none() {
            // ✅ RUNTIME WARNING - hier können wir versuchen zu übersetzen
            // Aber fallback auf English um Infinite Loops zu vermeiden
            log::warn!("Translation key not found: '{}'", key);
        }
        result
    }

    pub fn get_color_category_for_display(&self, display_category: &str) -> String {
        self.display_to_color_map
            .get(&display_category.to_lowercase())
            .cloned()
            .unwrap_or_else(|| display_category.to_lowercase())
    }
}
