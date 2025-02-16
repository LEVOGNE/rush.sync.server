// src/i18n/langs/mod.rs

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/i18n/langs/"]
pub struct Langs;

pub const AVAILABLE_LANGUAGES: &[&str] = &[
    "de", // Deutsch
];
pub const DEFAULT_LANGUAGE: &str = "de";

/* pub fn is_valid_language(lang: &str) -> bool {
    AVAILABLE_LANGUAGES
        .iter()
        .any(|&l| l == lang.to_lowercase())
}
 */

pub fn get_language_file(lang: &str) -> Option<&'static str> {
    match lang {
        "de" => Some(include_str!("de.json")),
        _ => None,
    }
}
