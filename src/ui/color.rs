// ui/color.rs - VEREINHEITLICHT UND PRAGMATISCH
use crate::core::prelude::*;
use log::Level;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    /// ✅ DIREKTE String-zu-Color Konvertierung - KEIN ENUM
    pub fn from_category_str(category: &str) -> Self {
        let color = match category.to_lowercase().as_str() {
            "error" => Color::Red,
            "warning" => Color::Yellow,
            "info" => Color::Green,
            "debug" => Color::Blue,
            "trace" => Color::DarkGray,
            "lang" => Color::Cyan,
            "version" => Color::LightBlue,
            _ => Color::Gray,
        };
        Self(color)
    }

    /// ✅ ALIAS für Kompatibilität mit altem Code
    pub fn from_custom_level(level: &str, _fallback_color: Option<u8>) -> Self {
        Self::from_category_str(level)
    }

    /// ✅ Standard Log-Level Support
    pub fn from_log_level(level: Level) -> Self {
        let color = match level {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Blue,
            Level::Trace => Color::DarkGray,
        };
        Self(color)
    }

    /// ✅ String-Parsing für Config-Dateien
    pub fn from_string(color_str: &str) -> crate::core::error::Result<Self> {
        let color = match color_str {
            "Black" => Color::Black,
            "Red" => Color::Red,
            "Green" => Color::Green,
            "Yellow" => Color::Yellow,
            "Blue" => Color::Blue,
            "Magenta" => Color::Magenta,
            "Cyan" => Color::Cyan,
            "Gray" => Color::Gray,
            "DarkGray" => Color::DarkGray,
            "LightRed" => Color::LightRed,
            "LightGreen" => Color::LightGreen,
            "LightYellow" => Color::LightYellow,
            "LightBlue" => Color::LightBlue,
            "LightMagenta" => Color::LightMagenta,
            "LightCyan" => Color::LightCyan,
            "White" => Color::White,
            _ => {
                return Err(AppError::Validation(format!(
                    "Ungültige Farbe: {}",
                    color_str
                )))
            }
        };
        Ok(Self(color))
    }

    /// ✅ ANSI-Formatierung für Terminal-Ausgabe
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

    /// ✅ ANSI-Color-Codes
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
            _ => 37, // Fallback: Gray
        }
    }

    /// ✅ String-Representation für Config-Export
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
            Color::Yellow => "Yellow",
            Color::Blue => "Blue",
            Color::Magenta => "Magenta",
            Color::Cyan => "Cyan",
            Color::Gray => "Gray",
            Color::DarkGray => "DarkGray",
            Color::LightRed => "LightRed",
            Color::LightGreen => "LightGreen",
            Color::LightYellow => "LightYellow",
            Color::LightBlue => "LightBlue",
            Color::LightMagenta => "LightMagenta",
            Color::LightCyan => "LightCyan",
            Color::White => "White",
            _ => "Gray",
        }
    }
}

// ✅ TRAIT-IMPLEMENTIERUNGEN
impl fmt::Display for AppColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
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
