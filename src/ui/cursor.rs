// ui/cursor.rs - KORRIGIERT: Robuste Bounds-Checks
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

pub struct CursorState {
    position: usize,
    text_length: usize,
    visible: bool,
    last_blink: Instant,
    blink_interval: Duration,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            position: 0,
            text_length: 0,
            visible: true,
            last_blink: Instant::now(),
            blink_interval: Duration::from_millis(333),
        }
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn update_blink(&mut self) {
        if self.last_blink.elapsed() >= self.blink_interval {
            self.visible = !self.visible;
            self.last_blink = Instant::now();
        }
    }

    pub fn show_cursor(&mut self) {
        self.visible = true;
        self.last_blink = Instant::now();
    }

    pub fn update_text_length(&mut self, text: &str) {
        self.text_length = text.graphemes(true).count();
        // ✅ KRITISCH: Position immer korrekt bounded
        self.position = self.position.min(self.text_length);
        self.show_cursor();
    }

    pub fn move_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.show_cursor();
        }
    }

    pub fn move_right(&mut self) {
        // ✅ ZUSÄTZLICHER CHECK: Nie über text_length hinaus
        if self.position < self.text_length {
            self.position += 1;
            self.show_cursor();
        }
    }

    pub fn move_to_start(&mut self) {
        self.position = 0;
        self.show_cursor();
    }

    pub fn move_to_end(&mut self) {
        self.position = self.text_length;
        self.show_cursor();
    }

    pub fn get_next_byte_position(&self, text: &str) -> usize {
        // ✅ SAFETY: Leerer Text check
        if text.is_empty() {
            return 0;
        }

        text.grapheme_indices(true)
            .take(self.position + 1)
            .last()
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(text.len())
    }

    pub fn get_byte_position(&self, text: &str) -> usize {
        // ✅ SAFETY: Mehrfache Checks
        if text.is_empty() || self.position == 0 {
            return 0;
        }

        // ✅ SAFETY: Position darf nie größer als text_length sein
        let safe_position = self.position.min(text.graphemes(true).count());
        if safe_position == 0 {
            return 0;
        }

        text.grapheme_indices(true)
            .nth(safe_position.saturating_sub(1))
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(text.len())
    }

    pub fn get_prev_byte_position(&self, text: &str) -> usize {
        // ✅ SAFETY: Umfassende Checks
        if text.is_empty() || self.position <= 1 {
            return 0;
        }

        // ✅ SAFETY: Position validieren
        let safe_position = self.position.min(text.graphemes(true).count());
        if safe_position <= 1 {
            return 0;
        }

        text.grapheme_indices(true)
            .nth(safe_position.saturating_sub(2))
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(0)
    }

    // ✅ NEU: Reset für leeren Text
    pub fn reset_for_empty_text(&mut self) {
        self.position = 0;
        self.text_length = 0;
        self.show_cursor();
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}
