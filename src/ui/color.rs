use crate::core::prelude::*;
use log::Level;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

static COLOR_MAP: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Standard colors
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

    // Categories
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

// Pre-compiled display text to color map for anti-flicker rendering
static DISPLAY_COLOR_MAP: LazyLock<HashMap<&'static str, Color>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Confirmations
    map.insert("CONFIRM", Color::Yellow);
    map.insert("BESTÄTIGEN", Color::Yellow);

    // Errors
    map.insert("ERROR", Color::Red);
    map.insert("FEHLER", Color::Red);
    map.insert("RENDER", Color::Red);

    // Warnings
    map.insert("WARN", Color::Yellow);
    map.insert("WARNING", Color::Yellow);
    map.insert("TERMINAL", Color::Yellow);

    // Info
    map.insert("INFO", Color::Green);
    map.insert("CLIPBOARD", Color::Green);
    map.insert("HISTORY", Color::Green);
    map.insert("HISTORIE", Color::Green);
    map.insert("LOG_LEVEL", Color::Green);
    map.insert("SYSTEM", Color::Green);

    // Debug
    map.insert("DEBUG", Color::Blue);

    // Trace
    map.insert("TRACE", Color::White);

    // Special categories
    map.insert("THEME", Color::LightMagenta);
    map.insert("LANG", Color::Cyan);
    map.insert("SPRACHE", Color::Cyan);
    map.insert("VERSION", Color::LightBlue);
    map.insert("READY", Color::Magenta);
    map.insert("BEREIT", Color::Magenta);

    map
});

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    /// O(1) lookup from display text to color, no computation needed.
    pub fn from_display_text(display_text: &str) -> Self {
        let normalized = display_text.trim().to_uppercase();
        let color = DISPLAY_COLOR_MAP
            .get(normalized.as_str())
            .copied()
            .unwrap_or(Color::Green); // info fallback

        Self(color)
    }

    pub fn from_category(category: &str) -> Self {
        let normalized = category.trim().to_lowercase();
        let color = COLOR_MAP
            .get(normalized.as_str())
            .copied()
            .unwrap_or(Color::Green);
        Self(color)
    }

    /// Legacy fallback: resolve color from any string key.
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

    /// Returns the resolved color along with the lookup duration for profiling.
    pub fn from_display_text_with_timing(display_text: &str) -> (Self, std::time::Duration) {
        let start = std::time::Instant::now();
        let color = Self::from_display_text(display_text);
        let duration = start.elapsed();
        (color, duration)
    }

    pub fn available_display_texts() -> Vec<&'static str> {
        DISPLAY_COLOR_MAP.keys().copied().collect()
    }

    pub fn available_categories() -> Vec<&'static str> {
        COLOR_MAP.keys().copied().collect()
    }

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
