// ## FILE: ui/widget.rs - UNVERÄNDERT
// ## BEGIN ##
use crate::core::prelude::*;
use ratatui::widgets::Paragraph;

pub trait Widget {
    fn render(&self) -> Paragraph;
    fn handle_input(&mut self, key: KeyEvent) -> Option<String>;
    fn as_input_state(&mut self) -> Option<&mut dyn InputWidget> {
        None
    }
}

pub trait InputWidget {
    fn update_cursor_blink(&mut self);
}
// ## END ##
