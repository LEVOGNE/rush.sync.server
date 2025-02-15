// src/i18n/langs/mod.rs

pub const AVAILABLE_LANGUAGES: [&str; 2] = ["de", "en"];
pub const DEFAULT_LANGUAGE: &str = "de";

/* pub fn is_valid_language(lang: &str) -> bool {
    AVAILABLE_LANGUAGES
        .iter()
        .any(|&l| l == lang.to_lowercase())
}
 */
pub fn get_language_file(lang: &str) -> Option<&'static str> {
    match lang.to_lowercase().as_str() {
        "de" => Some(include_str!("de.json")),
        "en" => Some(include_str!("en.json")),
        _ => None,
    }
}
