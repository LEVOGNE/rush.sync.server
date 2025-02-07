// src/color.rs
use crate::prelude::*;

use log::Level;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(pub Color);

impl AppColor {
    pub fn new(color: Color) -> Self {
        Self(color)
    }

    pub fn format_message(&self, level: &str, message: &str) -> String {
        format!(
            "\x1B[{}m[{}] {}\x1B[0m",
            self.to_ansi_code(),
            level,
            message
        )
    }

    pub fn from_log_level(level: Level) -> Self {
        match level {
            Level::Error => Self(Color::Red),
            Level::Warn => Self(Color::Yellow),
            Level::Info => Self(Color::Green),
            Level::Debug => Self(Color::Blue),
            Level::Trace => Self(Color::DarkGray),
        }
    }

    pub fn from_custom_level(level: &str) -> Self {
        match level {
            "LANG" => Self(Color::Cyan),
            "VERSION" => Self(Color::LightBlue),
            _ => Self(Color::Gray),
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
            _ => {
                log::debug!("Unrecognized color, falling back to gray (37)");
                37
            }
        }
    }

    // Parsing von String mit Fehlerbehandlung
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
}

// Implementierung der Standardfarben
impl Default for AppColor {
    fn default() -> Self {
        Self(Color::Gray)
    }
}

// Konvertierung für die Ratatui-Bibliothek
impl From<AppColor> for Color {
    fn from(app_color: AppColor) -> Self {
        app_color.0
    }
}

impl fmt::Display for AppColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
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
                _ => "Gray", // Standardfarbe
            }
        )
    }
}
