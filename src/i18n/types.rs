/* // src/i18n/types.rs
use crate::i18n::AppError;
use crate::i18n::TranslationError;
use crate::ui::color::ColorCategory;
use serde::{Deserialize, Serialize};

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
pub struct TranslationConfig {
    pub system: SystemTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemTranslations {
    pub startup: StartupTranslations,
    pub commands: CommandTranslations,
    pub log: LogTranslations,
    pub input: InputTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StartupTranslations {
    pub version: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandTranslations {
    pub unknown: TranslationEntry,
    pub language: LanguageTranslations,
    pub version: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LanguageTranslations {
    pub current: TranslationEntry,
    pub changed: TranslationEntry,
    pub invalid: TranslationEntry,
    pub available: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogTranslations {
    pub info: TranslationEntry,
    pub error: TranslationEntry,
    pub warn: TranslationEntry,
    pub debug: TranslationEntry,
    pub trace: TranslationEntry,
    pub language: TranslationEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputTranslations {
    pub confirm_exit: TranslationEntry,
    pub cancelled: TranslationEntry,
    pub confirm: InputConfirmTranslations,
    pub cancel: InputConfirmTranslations,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfirmTranslations {
    pub short: TranslationEntry,
}

// Default-Implementierungen
impl Default for SystemTranslations {
    fn default() -> Self {
        Self {
            startup: StartupTranslations::default(),
            commands: CommandTranslations::default(),
            log: LogTranslations::default(),
            input: InputTranslations::default(),
        }
    }
}

impl Default for StartupTranslations {
    fn default() -> Self {
        Self {
            version: TranslationEntry::default(),
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

impl Default for LogTranslations {
    fn default() -> Self {
        Self {
            info: TranslationEntry::default(),
            error: TranslationEntry::default(),
            warn: TranslationEntry::default(),
            debug: TranslationEntry::default(),
            trace: TranslationEntry::default(),
            language: TranslationEntry::default(),
        }
    }
}

impl Default for InputTranslations {
    fn default() -> Self {
        Self {
            confirm_exit: TranslationEntry::default(),
            cancelled: TranslationEntry::default(),
            confirm: InputConfirmTranslations::default(),
            cancel: InputConfirmTranslations::default(),
        }
    }
}

impl Default for InputConfirmTranslations {
    fn default() -> Self {
        Self {
            short: TranslationEntry::default(),
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            system: SystemTranslations::default(),
        }
    }
}

// Implementierung für Template-Zugriff
impl TranslationConfig {
    pub fn load(lang: &str) -> Result<Self, AppError> {
        let translation_str = match crate::i18n::langs::get_language_file(lang) {
            Some(content) => content,
            None => {
                return Err(AppError::Translation(TranslationError::LoadError(format!(
                    "Sprachdatei für '{}' nicht gefunden",
                    lang
                ))))
            }
        };

        match serde_json::from_str::<TranslationConfig>(translation_str) {
            Ok(config) => Ok(config),
            Err(e) => Err(AppError::Translation(TranslationError::LoadError(format!(
                "Fehler beim Parsen der Sprachdatei: {}",
                e
            )))),
        }
    }

    pub fn get_template(&self, key: &str) -> Option<(String, ColorCategory)> {
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
                "language" => Some((
                    self.system.log.language.text.clone(),
                    ColorCategory::from_str(&self.system.log.language.category),
                )),
                _ => None,
            },
            _ => None,
        }
    }
}
 */

// src/i18n/types.rs
use crate::i18n::{AppError, TranslationError};
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
        AppColor::new(ColorCategory::from_str(&self.category).to_color())
    }

    pub fn format(&self, params: &[&str]) -> (String, AppColor) {
        let text = if params.is_empty() {
            self.text.clone()
        } else {
            self.text.replace("{}", params[0])
        };

        (text, self.get_color())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationConfig {
    entries: HashMap<String, TranslationEntry>,
}

impl TranslationConfig {
    pub fn load(lang: &str) -> Result<Self, AppError> {
        let translation_str = crate::i18n::langs::get_language_file(lang).ok_or_else(|| {
            AppError::Translation(TranslationError::LoadError(format!(
                "Sprachdatei für '{}' nicht gefunden",
                lang
            )))
        })?;

        serde_json::from_str(translation_str).map_err(|e| {
            AppError::Translation(TranslationError::LoadError(format!(
                "Fehler beim Parsen der Sprachdatei: {}",
                e
            )))
        })
    }

    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        self.entries.get(key)
    }

    pub fn format(&self, key: &str, params: &[&str]) -> Option<(String, AppColor)> {
        self.get_entry(key).map(|entry| entry.format(params))
    }
}

impl Default for TranslationEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            category: "default".to_string(),
        }
    }
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}
