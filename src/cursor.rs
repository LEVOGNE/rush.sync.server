// src/cursor.rs
use unicode_segmentation::UnicodeSegmentation;

pub struct CursorState {
    position: usize,
    text_length: usize,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            position: 0,
            text_length: 0,
        }
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn update_text_length(&mut self, text: &str) {
        self.text_length = text.graphemes(true).count();
        self.position = self.position.min(self.text_length);
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

    pub fn move_to_start(&mut self) {
        self.position = 0;
    }

    pub fn move_to_end(&mut self) {
        self.position = self.text_length;
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
