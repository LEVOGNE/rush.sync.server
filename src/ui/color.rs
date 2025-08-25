// =====================================================
// ANTI-FLICKER COLOR SYSTEM: src/ui/color.rs
// =====================================================

use crate::core::prelude::*;
use log::Level;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

// ✅ BESTEHENDE COLOR_MAP (unverändert für Kategorien)
static COLOR_MAP: Lazy<HashMap<&'static str, Color>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Standard-Farben
    map.insert("black", Color::Black);
    map.insert("red", Color::Red);
    map.insert("green", Color::Green);
    map.insert("yellow", Color::Yellow);
    map.insert("blue", Color::Blue);
    map.insert("magenta", Color::Magenta);
    map.insert("cyan", Color::Cyan);
    map.insert("gray", Color::Gray);
    map.insert("darkgray", Color::DarkGray);
    map.insert("lightred", Color::LightRed);
    map.insert("lightgreen", Color::LightGreen);
    map.insert("lightyellow", Color::LightYellow);
    map.insert("lightblue", Color::LightBlue);
    map.insert("lightmagenta", Color::LightMagenta);
    map.insert("lightcyan", Color::LightCyan);
    map.insert("white", Color::White);

    // Kategorien
    map.insert("error", Color::Red);
    map.insert("warning", Color::Yellow);
    map.insert("warn", Color::Yellow);
    map.insert("info", Color::Green);
    map.insert("debug", Color::Blue);
    map.insert("trace", Color::White);
    map.insert("lang", Color::Cyan);
    map.insert("version", Color::LightBlue);
    map.insert("startup", Color::Magenta);
    map.insert("theme", Color::LightMagenta);

    map
});

// ✅ NEUER ANTI-FLICKER: PRE-COMPILED DISPLAY -> COLOR MAP
static DISPLAY_COLOR_MAP: Lazy<HashMap<&'static str, Color>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // ✅ ALLE DISPLAY-TEXTE DIREKT ZU FARBEN (ZERO DELAY!)

    // Bestätigungen → GELB
    map.insert("CONFIRM", Color::Yellow);
    map.insert("BESTÄTIGEN", Color::Yellow);

    // Fehler → ROT
    map.insert("ERROR", Color::Red);
    map.insert("FEHLER", Color::Red);
    map.insert("RENDER", Color::Red);

    // Warnungen → GELB
    map.insert("WARN", Color::Yellow);
    map.insert("WARNING", Color::Yellow);
    map.insert("TERMINAL", Color::Yellow);

    // Info → GRÜN
    map.insert("INFO", Color::Green);
    map.insert("CLIPBOARD", Color::Green);
    map.insert("HISTORY", Color::Green);
    map.insert("HISTORIE", Color::Green);
    map.insert("LOG_LEVEL", Color::Green);
    map.insert("SYSTEM", Color::Green);

    // Debug → BLAU
    map.insert("DEBUG", Color::Blue);

    // Trace → WEIß
    map.insert("TRACE", Color::White);

    // Spezielle → SPEZIALFARBEN
    map.insert("THEME", Color::LightMagenta); // PINK!
    map.insert("LANG", Color::Cyan); // CYAN
    map.insert("SPRACHE", Color::Cyan); // CYAN
    map.insert("VERSION", Color::LightBlue); // LIGHT_BLUE
    map.insert("READY", Color::Magenta); // MAGENTA
    map.insert("BEREIT", Color::Magenta); // MAGENTA

    map
});

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    // ✅ ANTI-FLICKER: ZERO-DELAY DISPLAY TEXT LOOKUP
    pub fn from_display_text(display_text: &str) -> Self {
        let normalized = display_text.trim().to_uppercase();

        // 🚀 DIREKT HIT: O(1) lookup, KEIN calculation, KEIN fallback!
        let color = DISPLAY_COLOR_MAP
            .get(normalized.as_str())
            .copied()
            .unwrap_or(Color::Green); // info fallback

        Self(color)
    }

    // ✅ PERFORMANCE-OPTIMIERT: Category lookup
    pub fn from_category(category: &str) -> Self {
        let normalized = category.trim().to_lowercase();
        let color = COLOR_MAP
            .get(normalized.as_str())
            .copied()
            .unwrap_or(Color::Green);
        Self(color)
    }

    // ✅ LEGACY SUPPORT: Vereinfacht für andere Stellen
    pub fn from_any<T: Into<String>>(source: T) -> Self {
        let key = source.into().to_lowercase();
        let color = COLOR_MAP.get(key.as_str()).copied().unwrap_or(Color::Green);
        Self(color)
    }

    pub fn from_log_level(level: Level) -> Self {
        Self::from_category(&level.to_string())
    }

    pub fn from_string(color_str: &str) -> crate::core::error::Result<Self> {
        let normalized = color_str.trim().to_lowercase();
        let color = COLOR_MAP
            .get(normalized.as_str())
            .copied()
            .ok_or_else(|| AppError::Validation(format!("Invalid color: {}", color_str)))?;
        Ok(Self(color))
    }

    // ✅ DEBUG: Performance monitoring
    pub fn from_display_text_with_timing(display_text: &str) -> (Self, std::time::Duration) {
        let start = std::time::Instant::now();
        let color = Self::from_display_text(display_text);
        let duration = start.elapsed();
        (color, duration)
    }

    // ✅ UTILITIES
    pub fn available_display_texts() -> Vec<&'static str> {
        DISPLAY_COLOR_MAP.keys().copied().collect()
    }

    pub fn available_categories() -> Vec<&'static str> {
        COLOR_MAP.keys().copied().collect()
    }

    // Bestehende Methoden...
    pub fn format_message(&self, level: &str, message: &str) -> String {
        if level.is_empty() {
            format!("\x1B[{}m{}\x1B[0m", self.to_ansi_code(), message)
        } else {
            format!(
                "\x1B[{}m[{}] {}\x1B[0m",
                self.to_ansi_code(),
                level,
                message
            )
        }
    }

    pub fn to_ansi_code(&self) -> u8 {
        match self.0 {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::Gray => 37,
            Color::DarkGray => 90,
            Color::LightRed => 91,
            Color::LightGreen => 92,
            Color::LightYellow => 93,
            Color::LightBlue => 94,
            Color::LightMagenta => 95,
            Color::LightCyan => 96,
            Color::White => 97,
            _ => 37,
        }
    }

    pub fn to_name(&self) -> &'static str {
        COLOR_MAP
            .iter()
            .find(|(_, &v)| v == self.0)
            .map(|(k, _)| *k)
            .unwrap_or("gray")
    }
}

// Traits unverändert...
impl From<AppColor> for Color {
    fn from(app_color: AppColor) -> Self {
        app_color.0
    }
}

impl From<&AppColor> for Color {
    fn from(app_color: &AppColor) -> Self {
        app_color.0
    }
}

impl Default for AppColor {
    fn default() -> Self {
        Self(Color::Gray)
    }
}
