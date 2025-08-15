// =====================================================
// FILE: src/ui/cursor.rs - OHNE DEBUG LOGS
// =====================================================

use crate::core::config::Config;
use crate::ui::color::AppColor;
use ratatui::prelude::{Span, Style};
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

/// Cursor-Typ unterscheidet wo der Cursor verwendet wird
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorKind {
    Input,  // Eingabebereich
    Output, // Ausgabebereich (Typewriter)
}

/// Cursor-Darstellung - einheitlich für beide Bereiche
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorType {
    Block,
    Pipe,
    Underscore,
}

// ✅ PROPER IMPLEMENTATION of FromStr trait
impl std::str::FromStr for CursorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BLOCK" => Ok(CursorType::Block),
            "PIPE" => Ok(CursorType::Pipe),
            "UNDERSCORE" => Ok(CursorType::Underscore),
            _ => Ok(CursorType::Pipe), // Default fallback
        }
    }
}

impl CursorType {
    // ✅ RENAMED to avoid confusion with FromStr::from_str
    pub fn parse_type(s: &str) -> CursorType {
        s.parse().unwrap_or(CursorType::Pipe)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            CursorType::Block => "█",
            CursorType::Pipe => "|",
            CursorType::Underscore => "_",
        }
    }
}

/// ✅ ZENTRALE CURSOR-IMPLEMENTIERUNG
/// Funktioniert für beide: Input & Output
#[derive(Debug, Clone)]
pub struct UiCursor {
    pub kind: CursorKind,
    pub ctype: CursorType,
    pub color: AppColor,
    pub fg: AppColor,
    pub position: usize,
    pub text_length: usize,
    pub blink_visible: bool,
    last_blink: Instant,
    blink_interval: Duration,
}

impl UiCursor {
    /// ✅ ZENTRALE FACTORY-METHODE - Erstellt Cursor basierend auf Config
    pub fn from_config(config: &Config, kind: CursorKind) -> Self {
        let (cursor_type_str, color, fg) = match kind {
            CursorKind::Input => (
                &config.theme.input_cursor,
                config.theme.input_cursor_color,
                config.theme.input_text,
            ),
            CursorKind::Output => (
                &config.theme.output_cursor,
                config.theme.output_cursor_color,
                config.theme.output_text,
            ),
        };

        let cursor_type = CursorType::parse_type(cursor_type_str);

        Self {
            kind,
            ctype: cursor_type,
            color,
            fg,
            position: 0,
            text_length: 0,
            blink_visible: true,
            last_blink: Instant::now(),
            blink_interval: Duration::from_millis(530),
        }
    }

    /// ✅ TYPEWRITER-FACTORY (Legacy-Support)
    pub fn for_typewriter() -> Self {
        Self {
            kind: CursorKind::Output,
            ctype: CursorType::Pipe,
            color: AppColor::default(),
            fg: AppColor::default(),
            position: 0,
            text_length: 0,
            blink_visible: true,
            last_blink: Instant::now(),
            blink_interval: Duration::from_millis(530),
        }
    }

    /// ✅ ZENTRALE CONFIG-UPDATE-METHODE
    pub fn update_from_config(&mut self, config: &Config) {
        let (cursor_type_str, color, fg) = match self.kind {
            CursorKind::Input => (
                &config.theme.input_cursor,
                config.theme.input_cursor_color,
                config.theme.input_text,
            ),
            CursorKind::Output => (
                &config.theme.output_cursor,
                config.theme.output_cursor_color,
                config.theme.output_text,
            ),
        };

        self.ctype = CursorType::parse_type(cursor_type_str);
        self.color = color;
        self.fg = fg;
    }

    /// ✅ NEUE METHODE: Update mit explizitem CursorKind (für Klarheit)
    pub fn update_from_config_explicit(&mut self, config: &Config, kind: CursorKind) {
        self.kind = kind;
        self.update_from_config(config);
    }

    // ==================== BLINK-VERWALTUNG ====================

    pub fn update_blink(&mut self) {
        if self.last_blink.elapsed() >= self.blink_interval {
            self.blink_visible = !self.blink_visible;
            self.last_blink = Instant::now();
        }
    }

    pub fn show_cursor(&mut self) {
        self.blink_visible = true;
        self.last_blink = Instant::now();
    }

    pub fn is_visible(&self) -> bool {
        self.blink_visible
    }

    // ==================== POSITION-VERWALTUNG ====================

    pub fn move_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.position < self.text_length {
            self.position += 1;
        }
    }

    pub fn move_to_start(&mut self) {
        self.position = 0;
    }

    pub fn move_to_end(&mut self) {
        self.position = self.text_length;
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn get_current_position(&self) -> usize {
        self.position
    }

    // ==================== TEXT-LÄNGEN-VERWALTUNG ====================

    pub fn update_text_length(&mut self, text: &str) {
        self.text_length = text.graphemes(true).count();
        if self.position > self.text_length {
            self.position = self.text_length;
        }
    }

    pub fn reset_for_empty_text(&mut self) {
        self.position = 0;
        self.text_length = 0;
    }

    // ==================== BYTE-POSITION FÜR TEXT-EDITING ====================

    pub fn get_byte_position(&self, text: &str) -> usize {
        text.grapheme_indices(true)
            .nth(self.position)
            .map(|(i, _)| i)
            .unwrap_or_else(|| text.len())
    }

    pub fn get_prev_byte_position(&self, text: &str) -> usize {
        if self.position == 0 {
            return 0;
        }
        text.grapheme_indices(true)
            .nth(self.position.saturating_sub(1))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    pub fn get_next_byte_position(&self, text: &str) -> usize {
        text.grapheme_indices(true)
            .nth(self.position + 1)
            .map(|(i, _)| i)
            .unwrap_or_else(|| text.len())
    }

    // ==================== RENDERING ====================

    /// ✅ BLOCK-CURSOR: Zeichen unter Cursor invertieren
    pub fn as_span(&self, text: &str, blink: bool) -> Span<'static> {
        if !blink || !self.blink_visible {
            let graphemes: Vec<&str> = text.graphemes(true).collect();
            let ch = graphemes.get(self.position).copied().unwrap_or(" ");
            return Span::styled(ch.to_string(), Style::default().fg(self.fg.into()));
        }

        // BLOCK: Zeichen unter Cursor invertieren
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let ch = graphemes.get(self.position).copied().unwrap_or(" ");
        Span::styled(
            ch.to_string(),
            Style::default().fg(self.fg.into()).bg(self.color.into()),
        )
    }

    /// ✅ CURSOR-SYMBOL-ERSTELLUNG für PIPE und UNDERSCORE
    pub fn create_cursor_span(&self, config: &Config) -> Span<'static> {
        let symbol = self.get_symbol();
        let cursor_color = self.color;

        let bg_color = match self.kind {
            CursorKind::Input => config.theme.input_bg.into(),
            CursorKind::Output => config.theme.output_bg.into(),
        };

        Span::styled(
            symbol.to_string(),
            Style::default().fg(cursor_color.into()).bg(bg_color),
        )
    }

    pub fn get_symbol(&self) -> &'static str {
        self.ctype.symbol()
    }

    // ==================== DEBUG & INFO ====================

    pub fn debug_info(&self) -> String {
        format!(
            "UiCursor({:?}): type={:?}, pos={}/{}, visible={}, symbol='{}', color='{}', fg='{}'",
            self.kind,
            self.ctype,
            self.position,
            self.text_length,
            self.blink_visible,
            self.get_symbol(),
            self.color.to_name(),
            self.fg.to_name()
        )
    }

    pub fn full_debug(&self) -> String {
        format!(
            "🔍 FULL CURSOR DEBUG:\n\
            Kind: {:?}\n\
            Type: {:?}\n\
            Symbol: '{}'\n\
            Cursor Color: '{}'\n\
            Text Color: '{}'\n\
            Position: {}/{}\n\
            Visible: {}",
            self.kind,
            self.ctype,
            self.get_symbol(),
            self.color.to_name(),
            self.fg.to_name(),
            self.position,
            self.text_length,
            self.blink_visible,
        )
    }

    pub fn detailed_debug(&self) -> String {
        format!(
            "🔍 DETAILED CURSOR DEBUG:\n\
            🏷️ Kind: {:?}\n\
            🎯 Type: {:?} (symbol: '{}')\n\
            🎨 Cursor Color: '{}' ⬅️ IST DAS RICHTIG?\n\
            🎨 Text Color (fg): '{}'\n\
            📍 Position: {}/{}\n\
            👁️ Visible: {}\n\
            ⏱️ Last Blink: {:?}",
            self.kind,
            self.ctype,
            self.get_symbol(),
            self.color.to_name(), // ⬅️ Das sollte "lightblue" sein!
            self.fg.to_name(),
            self.position,
            self.text_length,
            self.blink_visible,
            self.last_blink.elapsed()
        )
    }
}

// ==================== FACTORY-FUNKTIONEN ====================

pub fn create_input_cursor(config: &Config) -> UiCursor {
    UiCursor::from_config(config, CursorKind::Input)
}

pub fn create_output_cursor(config: &Config) -> UiCursor {
    UiCursor::from_config(config, CursorKind::Output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_types() {
        assert_eq!(CursorType::parse_type("BLOCK").symbol(), "█");
        assert_eq!(CursorType::parse_type("PIPE").symbol(), "|");
        assert_eq!(CursorType::parse_type("UNDERSCORE").symbol(), "_");
        assert_eq!(CursorType::parse_type("unknown").symbol(), "|"); // Fallback to PIPE
    }

    #[test]
    fn test_fromstr_trait() {
        assert_eq!("BLOCK".parse::<CursorType>().unwrap(), CursorType::Block);
        assert_eq!("PIPE".parse::<CursorType>().unwrap(), CursorType::Pipe);
        assert_eq!(
            "UNDERSCORE".parse::<CursorType>().unwrap(),
            CursorType::Underscore
        );
        assert_eq!("unknown".parse::<CursorType>().unwrap(), CursorType::Pipe); // Fallback
    }

    #[test]
    fn test_cursor_position() {
        let config = crate::core::config::Config::default();
        let mut cursor = UiCursor::from_config(&config, CursorKind::Input);

        cursor.update_text_length("hello");
        assert_eq!(cursor.text_length, 5);

        cursor.move_right();
        cursor.move_right();
        assert_eq!(cursor.position, 2);

        cursor.move_to_end();
        assert_eq!(cursor.position, 5);

        cursor.move_to_start();
        assert_eq!(cursor.position, 0);
    }

    #[test]
    fn test_input_cursor_color() {
        let config = crate::core::config::Config::default();
        let cursor = UiCursor::from_config(&config, CursorKind::Input);

        assert_eq!(
            cursor.color.to_name(),
            config.theme.input_cursor_color.to_name()
        );
    }
}
