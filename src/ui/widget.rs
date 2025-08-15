use crate::core::prelude::*;
use crate::input::state::InputStateBackup;
use ratatui::widgets::Paragraph;

pub trait Widget {
    fn render(&self) -> Paragraph;

    fn render_with_cursor(&self) -> (Paragraph, Option<(u16, u16)>) {
        (self.render(), None) // Default: Kein Terminal-Cursor
    }

    fn handle_input(&mut self, key: KeyEvent) -> Option<String>;
    fn as_input_state(&mut self) -> Option<&mut dyn InputWidget> {
        None
    }

    /// ✅ NEU: Get backup data (default: empty)
    fn get_backup_data(&self) -> Option<InputStateBackup> {
        None
    }

    /// ✅ NEU: Restore from backup data (default: do nothing)
    fn restore_backup_data(&mut self, _backup: InputStateBackup) {
        // Default implementation: do nothing
    }
}

pub trait InputWidget {
    fn update_cursor_blink(&mut self);
}
