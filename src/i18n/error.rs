// src/i18n/error.rs

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
