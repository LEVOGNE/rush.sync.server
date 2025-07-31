// =====================================================
// FILE: src/ui/cursor.rs - FIXED INPUT CURSOR IMPLEMENTATION
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
    Block,      // █
    Pipe,       // |
    Underscore, // _
    Default,    // ✅ FIXED: Jetzt wird das korrekte Symbol verwendet
}

impl CursorType {
    pub fn from_str(s: &str) -> CursorType {
        let result = match s.to_uppercase().as_str() {
            "BLOCK" => CursorType::Block,
            "PIPE" => CursorType::Pipe,
            "UNDERSCORE" => CursorType::Underscore,
            _ => CursorType::Default, // ✅ FIXED: Default wird korrekt behandelt
        };

        // ✅ DEBUG für Parsing-Verifizierung
        if std::env::var("RUST_LOG")
            .unwrap_or_default()
            .contains("debug")
        {
            eprintln!("🔍 CursorType::from_str('{}') → {:?}", s, result);
        }

        result
    }

    pub fn symbol(self) -> &'static str {
        match self {
            CursorType::Block => "█",
            CursorType::Pipe => "|",
            CursorType::Underscore => "_",
            CursorType::Default => "|", // ✅ FIXED: Default = PIPE Symbol statt "///"
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
                config.theme.input_cursor_color, // ✅ FIXED: Richtige Farbe für Input-Cursor
                config.theme.input_text,
            ),
            CursorKind::Output => (
                &config.theme.output_cursor,
                config.theme.output_cursor_color,
                config.theme.output_text,
            ),
        };

        // ✅ DEBUG: Verifiziere Config-Werte
        if std::env::var("RUST_LOG")
            .unwrap_or_default()
            .contains("debug")
        {
            eprintln!(
                "🔧 UiCursor::from_config({:?}): type='{}', color='{}', fg='{}'",
                kind,
                cursor_type_str,
                color.to_name(),
                fg.to_name()
            );
        }

        Self {
            kind,
            ctype: CursorType::from_str(cursor_type_str),
            color, // ✅ FIXED: Jetzt wird die richtige Farbe verwendet
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
            ctype: CursorType::Default,
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
                config.theme.input_cursor_color, // ✅ FIXED: Richtige Farbe
                config.theme.input_text,
            ),
            CursorKind::Output => (
                &config.theme.output_cursor,
                config.theme.output_cursor_color,
                config.theme.output_text,
            ),
        };

        self.ctype = CursorType::from_str(cursor_type_str);
        self.color = color; // ✅ FIXED: Jetzt wird die richtige Farbe gesetzt
        self.fg = fg;

        log::debug!(
            "🔄 Cursor updated: {:?} → type='{}', color='{}', symbol='{}'",
            self.kind,
            cursor_type_str,
            color.to_name(),
            self.get_symbol()
        );
    }

    // ==================== BLINK-VERWALTUNG ====================

    /// ✅ FIXED: Blink-Status aktualisieren
    pub fn update_blink(&mut self) {
        if self.last_blink.elapsed() >= self.blink_interval {
            self.blink_visible = !self.blink_visible;
            self.last_blink = Instant::now();
        }
    }

    /// Cursor sichtbar machen (nach Screen-Clear)
    pub fn show_cursor(&mut self) {
        self.blink_visible = true;
        self.last_blink = Instant::now();
    }

    pub fn is_visible(&self) -> bool {
        self.blink_visible
    }

    // ==================== POSITION-VERWALTUNG ====================

    pub fn move_to_start(&mut self) {
        self.position = 0;
    }

    pub fn move_to_end(&mut self) {
        self.position = self.text_length;
    }

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

    /// ✅ FIXED: INTELLIGENTE SPAN-ERSTELLUNG für Input-Bereich
    /// Nur für BLOCK-Cursor (wird nur noch für Block verwendet)
    pub fn as_span(&self, text: &str, blink: bool) -> Span<'static> {
        if !blink || !self.blink_visible {
            // Cursor nicht sichtbar → normales Zeichen anzeigen
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

    /// ✅ ZENTRALE CURSOR-SYMBOL-ERSTELLUNG
    /// Rendert den Cursor als separates Symbol (für Output-Bereich UND Input-non-BLOCK)
    pub fn create_cursor_span(&self, config: &Config) -> Span<'static> {
        let symbol = self.get_symbol();

        Span::styled(
            symbol.to_string(),
            Style::default()
                .fg(self.color.into()) // ✅ FIXED: Richtige Cursor-Farbe
                .bg(config.theme.input_bg.into()), // ✅ FIXED: Input-Hintergrund für Input-Cursor
        )
    }

    pub fn get_symbol(&self) -> &'static str {
        self.ctype.symbol()
    }

    // ==================== DEBUG & INFO ====================

    pub fn debug_info(&self) -> String {
        format!(
            "UiCursor({:?}): type={}, pos={}/{}, visible={}, symbol='{}', color='{}'",
            self.kind,
            match self.ctype {
                CursorType::Block => "BLOCK",
                CursorType::Pipe => "PIPE",
                CursorType::Underscore => "UNDERSCORE",
                CursorType::Default => "DEFAULT",
            },
            self.position,
            self.text_length,
            self.blink_visible,
            self.get_symbol(),
            self.color.to_name()
        )
    }
}

// ==================== FACTORY-FUNKTIONEN ====================

/// ✅ Erstelle Input-Cursor
pub fn create_input_cursor(config: &Config) -> UiCursor {
    UiCursor::from_config(config, CursorKind::Input)
}

/// ✅ Erstelle Output-Cursor
pub fn create_output_cursor(config: &Config) -> UiCursor {
    UiCursor::from_config(config, CursorKind::Output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_types() {
        assert_eq!(CursorType::from_str("BLOCK").symbol(), "█");
        assert_eq!(CursorType::from_str("PIPE").symbol(), "|");
        assert_eq!(CursorType::from_str("UNDERSCORE").symbol(), "_");
        assert_eq!(CursorType::from_str("DEFAULT").symbol(), "|"); // ✅ FIXED
        assert_eq!(CursorType::from_str("unknown").symbol(), "|"); // ✅ FIXED
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

        // ✅ FIXED: Test dass Input-Cursor die richtige Farbe bekommt
        assert_eq!(
            cursor.color.to_name(),
            config.theme.input_cursor_color.to_name()
        );
    }
}
