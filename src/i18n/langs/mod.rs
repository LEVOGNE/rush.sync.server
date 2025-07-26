// src/i18n/langs/mod.rs - ENGLISH SUPPORT HINZUGEFÜGT

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/i18n/langs/"]
pub struct Langs;

pub const AVAILABLE_LANGUAGES: &[&str] = &[
    "de", // Deutsch
    "en", // English
];
pub const DEFAULT_LANGUAGE: &str = "en";

pub fn get_language_file(lang: &str) -> Option<&'static str> {
    match lang {
        "de" => Some(include_str!("de.json")),
        "en" => Some(include_str!("en.json")),
        _ => None,
    }
}
