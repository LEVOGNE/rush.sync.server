use crate::prelude::*;

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
