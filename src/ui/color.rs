use crate::core::prelude::*;
use log::Level;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

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

    map
});

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    // pub fn from_any<T: Into<String>>(source: T) -> Self {
    //     let key = source.into().to_lowercase();
    //     Self(*COLOR_MAP.get(key.as_str()).unwrap_or(&Color::Gray))
    // }

    pub fn from_any<T: Into<String>>(source: T) -> Self {
        let key = source.into().to_lowercase();

        // ✅ DIRECT LOOKUP FIRST (für echte Farbnamen)
        if let Some(&color) = COLOR_MAP.get(key.as_str()) {
            log::debug!("✅ Direct color lookup: '{}' → {:?}", key, color);
            return Self(color);
        }

        // ❌ FALLBACK: i18n category mapping (nur für Display-Kategorien)
        log::debug!("⚠️ Using i18n category mapping for: '{}'", key);
        let mapped_category = crate::i18n::get_color_category_for_display(&key);
        let fallback_color = COLOR_MAP
            .get(mapped_category.as_str())
            .unwrap_or(&Color::Gray);

        log::debug!(
            "📍 Category mapping: '{}' → '{}' → {:?}",
            key,
            mapped_category,
            fallback_color
        );
        Self(*fallback_color)
    }

    pub fn from_log_level(level: Level) -> Self {
        Self::from_any(level.to_string())
    }

    pub fn from_string(color_str: &str) -> crate::core::error::Result<Self> {
        let normalized = color_str.trim().to_lowercase();

        // ✅ NUR DEBUG-LEVEL (nicht INFO)
        log::debug!(
            "🎨 AppColor::from_string: '{}' → normalized: '{}'",
            color_str,
            normalized
        );

        // ✅ DIRECT LOOKUP: Gehe DIREKT zu COLOR_MAP, nicht über i18n!
        let color = COLOR_MAP.get(normalized.as_str()).copied().ok_or_else(|| {
            log::error!("❌ Color '{}' not found in COLOR_MAP", normalized);
            log::debug!(
                "📋 Available colors: {:?}",
                COLOR_MAP.keys().collect::<Vec<_>>()
            );
            AppError::Validation(format!("Invalid color: {}", color_str))
        })?;

        let app_color = Self(color);

        // ✅ NUR DEBUG-LEVEL (nicht INFO) - viel weniger Spam
        log::debug!(
            "✅ Color '{}' → '{}' → RGB({:?})",
            color_str,
            app_color.to_name(),
            color
        );

        Ok(app_color)
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
