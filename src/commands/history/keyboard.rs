use crate::core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum HistoryAction {
    NavigatePrevious,
    NavigateNext,
}

pub struct HistoryKeyboardHandler;

impl HistoryKeyboardHandler {
    pub fn get_history_action(key: &KeyEvent) -> Option<HistoryAction> {
        match (key.code, key.modifiers) {
            (KeyCode::Up, crossterm::event::KeyModifiers::NONE) => {
                Some(HistoryAction::NavigatePrevious)
            }
            (KeyCode::Down, crossterm::event::KeyModifiers::NONE) => {
                Some(HistoryAction::NavigateNext)
            }
            _ => None,
        }
    }
}
