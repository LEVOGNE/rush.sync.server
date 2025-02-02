// ## FILE: ./src/widget.rs
use crate::prelude::*;

pub trait Widget {
    fn render(&self) -> Paragraph;
    fn handle_input(&mut self, key: KeyEvent) -> Option<String>;
}
