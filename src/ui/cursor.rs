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
            blink_interval: Duration::from_millis(530), // Standard Blink-Interval
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
        self.position = self.position.min(self.text_length);
        self.show_cursor(); // Cursor bei Texteingabe sichtbar machen
    }

    pub fn move_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.show_cursor();
        }
    }

    pub fn move_right(&mut self) {
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

    pub fn get_byte_position(&self, text: &str) -> usize {
        text.grapheme_indices(true)
            .take(self.position)
            .last()
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(0)
    }

    pub fn get_next_byte_position(&self, text: &str) -> usize {
        text.grapheme_indices(true)
            .take(self.position + 1)
            .last()
            .map(|(pos, grapheme)| pos + grapheme.len())
            .unwrap_or(text.len())
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}
