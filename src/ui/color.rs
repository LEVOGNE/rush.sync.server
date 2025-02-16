use crate::i18n;
use crate::prelude::*;
use log::Level;
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
    pub fn to_color(&self) -> AppColor {
        match self {
            ColorCategory::Error => AppColor::new(Color::Red),
            ColorCategory::Warning => AppColor::new(Color::Yellow),
            ColorCategory::Info => AppColor::new(Color::Green),
            ColorCategory::Debug => AppColor::new(Color::Blue),
            ColorCategory::Trace => AppColor::new(Color::DarkGray),
            ColorCategory::Language => AppColor::new(Color::Cyan),
            ColorCategory::Version => AppColor::new(Color::LightBlue),
            ColorCategory::Default => AppColor::new(Color::Gray),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => ColorCategory::Error,
            "warning" => ColorCategory::Warning,
            "info" => ColorCategory::Info,
            "debug" => ColorCategory::Debug,
            "trace" => ColorCategory::Trace,
            "language" => ColorCategory::Language,
            "version" => ColorCategory::Version,
            _ => ColorCategory::Default,
        }
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

    // Neue Methode zum Parsen von ANSI-Codes
    pub fn from_ansi_code(code: u8) -> Self {
        match code {
            30 => Self(Color::Black),
            31 => Self(Color::Red),
            32 => Self(Color::Green),
            33 => Self(Color::Yellow),
            34 => Self(Color::Blue),
            35 => Self(Color::Magenta),
            36 => Self(Color::Cyan),
            37 => Self(Color::Gray),
            90 => Self(Color::DarkGray),
            91 => Self(Color::LightRed),
            92 => Self(Color::LightGreen),
            93 => Self(Color::LightYellow),
            94 => Self(Color::LightBlue),
            95 => Self(Color::LightMagenta),
            96 => Self(Color::LightCyan),
            97 => Self(Color::White),
            _ => Self(Color::Gray), // Fallback
        }
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

    pub fn from_custom_level(level: &str, fallback_color: Option<u8>) -> Self {
        // Mapping für verschiedene Fehler-Level-Bezeichnungen
        let error_translations = vec![
            i18n::get_translation("system.log.error", &[]).to_uppercase(),
            "ERROR".to_string(),
            "[ERROR]".to_string(),
        ];

        let warning_translations = vec![
            i18n::get_translation("system.log.warn", &[]).to_uppercase(),
            "WARNING".to_string(),
            "[WARNING]".to_string(),
        ];

        let info_translations = vec![
            i18n::get_translation("system.log.info", &[]).to_uppercase(),
            "INFO".to_string(),
            "[INFO]".to_string(),
        ];

        let debug_translations = vec![
            i18n::get_translation("system.log.debug", &[]).to_uppercase(),
            "DEBUG".to_string(),
            "[DEBUG]".to_string(),
        ];

        let trace_translations = vec![
            i18n::get_translation("system.log.trace", &[]).to_uppercase(),
            "TRACE".to_string(),
            "[TRACE]".to_string(),
        ];

        // Prüfe den übergebenen Level gegen die Übersetzungen
        let normalized_level = level.to_uppercase();

        // Primäre Übersetzungs-Logik
        if error_translations.contains(&normalized_level) {
            return Self(Color::Red);
        } else if warning_translations.contains(&normalized_level) {
            return Self(Color::Yellow);
        } else if info_translations.contains(&normalized_level) {
            return Self(Color::Green);
        } else if debug_translations.contains(&normalized_level) {
            return Self(Color::Blue);
        } else if trace_translations.contains(&normalized_level) {
            return Self(Color::DarkGray);
        }

        // Fallback mit ANSI-Code, wenn bereitgestellt
        if let Some(ansi_code) = fallback_color {
            return Self::from_ansi_code(ansi_code);
        }

        // Finale Fallback-Logik
        match normalized_level.as_str() {
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

    pub fn from_category(category: ColorCategory) -> Self {
        category.to_color()
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
