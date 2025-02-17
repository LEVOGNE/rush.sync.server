// src/ui/color.rs
use crate::prelude::*;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorCategory {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
    Language,
    Version,
    Default,
}

impl ColorCategory {
    pub fn to_color(&self) -> Color {
        match self {
            ColorCategory::Error => Color::Red,
            ColorCategory::Warning => Color::Yellow,
            ColorCategory::Info => Color::Green,
            ColorCategory::Debug => Color::Blue,
            ColorCategory::Trace => Color::DarkGray,
            ColorCategory::Language => Color::Cyan,
            ColorCategory::Version => Color::LightBlue,
            ColorCategory::Default => Color::Gray,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            "language" => Self::Language,
            "version" => Self::Version,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    // Neue Methode für Kategorie-Konvertierung
    pub fn from_category(category: ColorCategory) -> Self {
        Self(category.to_color())
    }

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

    pub fn format_message(&self, level: &str, message: &str) -> String {
        format!(
            "\x1B[{}m[{}] {}\x1B[0m",
            self.to_ansi_code(),
            level,
            message
        )
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

    pub fn from_string(color_str: &str) -> Result<Self> {
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

    pub fn from_log_level(level: Level) -> Self {
        let category = match level {
            Level::Error => ColorCategory::Error,
            Level::Warn => ColorCategory::Warning,
            Level::Info => ColorCategory::Info,
            Level::Debug => ColorCategory::Debug,
            Level::Trace => ColorCategory::Trace,
        };
        Self::from_category(category)
    }

    pub fn from_custom_level(level: &str, _fallback_color: Option<u8>) -> Self {
        // Konvertiere den Level-String in eine ColorCategory
        Self::from_category(ColorCategory::from_str(level))
    }
}

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

impl Default for AppColor {
    fn default() -> Self {
        Self(Color::Gray)
    }
}

impl fmt::Display for ColorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ColorCategory::Error => "ERROR",
                ColorCategory::Warning => "WARNING",
                ColorCategory::Info => "INFO",
                ColorCategory::Debug => "DEBUG",
                ColorCategory::Trace => "TRACE",
                ColorCategory::Language => "LANG",
                ColorCategory::Version => "VERSION",
                ColorCategory::Default => "INFO",
            }
        )
    }
}
