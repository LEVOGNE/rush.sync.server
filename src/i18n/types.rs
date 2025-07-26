// i18n/types.rs - ENHANCED JSON LOADING FIX
use crate::core::error::{AppError, Result};
use crate::i18n::error::TranslationError;
use crate::ui::color::AppColor;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TranslationEntry {
    pub text: String,
    pub color_category: String, // ✅ Für Farbe (intern: "error", "warning", etc.)
    pub display_category: String, // ✅ Für Anzeige (übersetzt: "fehler", "warnung", etc.)
}

impl TranslationEntry {
    pub fn get_color(&self) -> AppColor {
        // ✅ Farbe basiert auf color_category (immer englisch)
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

    // ✅ COMMAND-SYSTEM: Verwendet display_category für Anzeige
    pub fn format_for_command(&self, params: &[&str]) -> String {
        let (text, _color) = self.format(params);
        // ✅ Verwende display_category für Anzeige (übersetzt)
        format!("[{}] {}", self.display_category.to_uppercase(), text)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TranslationConfig {
    entries: HashMap<String, TranslationEntry>,
    // ✅ DYNAMISCHES REVERSE-MAPPING: display_category -> color_category
    display_to_color_map: HashMap<String, String>,
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

        // ✅ ERWEITERTE KONVERTIERUNG mit flexiblem Schema
        let mut entries = HashMap::new();
        let mut display_to_color_map = HashMap::new();
        let mut processed_count = 0;
        let mut skipped_count = 0;

        for (key, value) in raw_entries.iter() {
            if key.ends_with(".text") {
                let base_key = &key[0..key.len() - 5]; // Remove ".text"

                // ✅ FLEXIBLES SCHEMA: Verschiedene Varianten unterstützen
                let (color_category, display_category) =
                    Self::determine_categories(&raw_entries, base_key);

                if color_category.is_none() {
                    log::warn!("⚠️  No category found for key: {}", base_key);
                    skipped_count += 1;
                    continue;
                }

                let color_cat = color_category.unwrap();
                let display_cat = display_category.unwrap_or_else(|| color_cat.clone());

                log::debug!(
                    "✅ Processing: {} -> text: '{}', color: '{}', display: '{}'",
                    base_key,
                    value.chars().take(30).collect::<String>(),
                    color_cat,
                    display_cat
                );

                entries.insert(
                    base_key.to_string(),
                    TranslationEntry {
                        text: value.clone(),
                        color_category: color_cat.clone(),
                        display_category: display_cat.clone(),
                    },
                );

                // ✅ REVERSE-MAPPING erstellen (case-insensitive)
                display_to_color_map.insert(display_cat.to_lowercase(), color_cat.to_lowercase());

                processed_count += 1;
            }
        }

        log::info!(
            "🎯 Enhanced translation processing: {} entries processed, {} skipped",
            processed_count,
            skipped_count
        );

        Ok(Self {
            entries,
            display_to_color_map,
        })
    }

    /// ✅ FLEXIBLES SCHEMA: Unterstützt verschiedene JSON-Formate
    fn determine_categories(
        raw_entries: &HashMap<String, String>,
        base_key: &str,
    ) -> (Option<String>, Option<String>) {
        // Schema 1: Neue Syntax mit separaten Kategorien
        // .color_category + .display_category
        let color_category_key = format!("{}.color_category", base_key);
        let display_category_key = format!("{}.display_category", base_key);

        if let (Some(color_cat), Some(display_cat)) = (
            raw_entries.get(&color_category_key),
            raw_entries.get(&display_category_key),
        ) {
            log::debug!("📋 Schema 1: Separate categories for {}", base_key);
            return (Some(color_cat.clone()), Some(display_cat.clone()));
        }

        // Schema 2: Gemischte Syntax (DEIN FORMAT!)
        // .category (für Farbe) + .display_category (für Anzeige)
        let category_key = format!("{}.category", base_key);

        if let (Some(color_cat), Some(display_cat)) = (
            raw_entries.get(&category_key),
            raw_entries.get(&display_category_key),
        ) {
            log::debug!(
                "📋 Schema 2: Mixed categories for {} (color: {}, display: {})",
                base_key,
                color_cat,
                display_cat
            );
            return (Some(color_cat.clone()), Some(display_cat.clone()));
        }

        // Schema 3: Legacy Syntax
        // .category (für beide)
        if let Some(legacy_cat) = raw_entries.get(&category_key) {
            log::debug!(
                "📋 Schema 3: Legacy category for {}: {}",
                base_key,
                legacy_cat
            );
            return (Some(legacy_cat.clone()), Some(legacy_cat.clone()));
        }

        // Nichts gefunden
        log::debug!("❌ No valid category schema found for {}", base_key);
        (None, None)
    }

    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        let result = self.entries.get(key);
        if result.is_none() {
            log::warn!("🔍 Translation key not found in config: '{}'", key);
        }
        result
    }

    /// ✅ DYNAMISCHES MAPPING: display_category -> color_category
    pub fn get_color_category_for_display(&self, display_category: &str) -> String {
        self.display_to_color_map
            .get(&display_category.to_lowercase())
            .cloned()
            .unwrap_or_else(|| {
                // Fallback: Verwende display_category als color_category
                log::debug!(
                    "No mapping found for display category '{}', using as-is",
                    display_category
                );
                display_category.to_lowercase()
            })
    }
}
