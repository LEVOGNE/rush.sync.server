use crate::core::prelude::*;
use log::Level;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppColor(Color);

impl AppColor {
    pub fn from_any<T: Into<String>>(source: T) -> Self {
        Self::from_category(&source.into())
    }

    pub fn new(color: Color) -> Self {
        Self(color)
    }

    pub fn from_category_str(category: &str) -> Self {
        Self::from_category(category)
    }

    pub fn from_string(color_str: &str) -> crate::core::error::Result<Self> {
        let color = match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "darkgray" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
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
        Self::from_category(&level.to_string().to_lowercase())
    }

    // ✅ ZENTRALE Kategorisierung - NUR COLOR_CATEGORIES!
    fn from_category(category: &str) -> Self {
        let color = match category.to_lowercase().as_str() {
            "error" => Color::Red,
            "warning" | "warn" => Color::Yellow,
            "info" => Color::Green,
            "debug" => Color::Blue,
            "trace" => Color::DarkGray,
            "lang" => Color::Cyan, // ✅ NUR EINE COLOR-CATEGORY!
            "version" => Color::LightBlue,
            _ => Color::Gray,
        };
        Self(color)
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
