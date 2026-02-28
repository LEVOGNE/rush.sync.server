// src/ui/widget.rs
use crate::core::prelude::*;
use crate::input::state::InputStateBackup;
use ratatui::widgets::Paragraph;

/// Core widget trait for rendering
pub trait Widget {
    fn render(&self) -> Paragraph<'_>;
    fn handle_input(&mut self, key: KeyEvent) -> Option<String>;
}

/// Widgets that support cursor rendering
pub trait CursorWidget: Widget {
    fn render_with_cursor(&self) -> (Paragraph<'_>, Option<(u16, u16)>);
}

/// State management for widgets
pub trait StatefulWidget<T = InputStateBackup> {
    fn export_state(&self) -> T;
    fn import_state(&mut self, state: T);
}

/// Animated widgets (blinking, etc.)
pub trait AnimatedWidget {
    fn tick(&mut self);
}

/// Full input widget (combines all traits)
pub trait InputWidget: Widget + CursorWidget + StatefulWidget + AnimatedWidget {}

// Blanket Implementation
impl<T> InputWidget for T where T: Widget + CursorWidget + StatefulWidget + AnimatedWidget {}

/// Widget Utilities
pub mod utils {
    use super::*;
    use ratatui::{
        style::Style,
        widgets::{Block, Borders},
    };

    pub fn simple_text(content: &str, style: Style) -> Paragraph<'_> {
        Paragraph::new(content.to_string())
            .style(style)
            .block(Block::default().borders(Borders::NONE))
    }

    pub fn has_cursor<T: Widget>(_: &T) -> bool {
        std::any::type_name::<T>().contains("CursorWidget")
    }
}

/// Example implementations for tests
#[cfg(test)]
mod examples {
    use super::*;

    #[derive(Debug)]
    pub struct SimpleWidget(String);

    impl Widget for SimpleWidget {
        fn render(&self) -> Paragraph<'_> {
            utils::simple_text(&self.0, ratatui::style::Style::default())
        }

        fn handle_input(&mut self, _: KeyEvent) -> Option<String> {
            None
        }
    }

    #[derive(Debug)]
    pub struct FullInputWidget {
        content: String,
        cursor_pos: usize,
        visible: bool,
    }

    impl Widget for FullInputWidget {
        fn render(&self) -> Paragraph<'_> {
            self.render_with_cursor().0
        }

        fn handle_input(&mut self, _: KeyEvent) -> Option<String> {
            Some("handled".to_string())
        }
    }

    impl CursorWidget for FullInputWidget {
        fn render_with_cursor(&self) -> (Paragraph<'_>, Option<(u16, u16)>) {
            let para = utils::simple_text(&self.content, ratatui::style::Style::default());
            let cursor = if self.visible {
                Some((self.cursor_pos as u16, 0))
            } else {
                None
            };
            (para, cursor)
        }
    }

    impl StatefulWidget for FullInputWidget {
        fn export_state(&self) -> InputStateBackup {
            InputStateBackup {
                content: self.content.clone(),
                history: vec![],
                cursor_pos: self.cursor_pos,
            }
        }

        fn import_state(&mut self, state: InputStateBackup) {
            self.content = state.content;
            self.cursor_pos = state.cursor_pos;
        }
    }

    impl AnimatedWidget for FullInputWidget {
        fn tick(&mut self) {
            self.visible = !self.visible;
        }
    }

    #[test]
    fn test_widget_system() {
        let mut simple = SimpleWidget("test".to_string());
        let _para = simple.render();
        assert_eq!(simple.handle_input(KeyEvent::from(KeyCode::Enter)), None);

        let mut full = FullInputWidget {
            content: "input".to_string(),
            cursor_pos: 5,
            visible: true,
        };

        // Test all traits
        let _para = full.render();
        let (_para, cursor) = full.render_with_cursor();
        assert_eq!(cursor, Some((5, 0)));

        let state = full.export_state();
        full.content = "changed".to_string();
        full.import_state(state);
        assert_eq!(full.content, "input");

        let old_visible = full.visible;
        full.tick();
        assert_ne!(full.visible, old_visible);
    }
}

/// Migration helper for existing code
pub mod compat {
    pub use super::{
        AnimatedWidget as Tickable, CursorWidget as RenderWithCursor,
        InputWidget as InputWidgetFull, StatefulWidget as Stateful, Widget,
    };
}
