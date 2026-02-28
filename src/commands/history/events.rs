#[derive(Debug, Clone, PartialEq)]
pub enum HistoryEvent {
    Clear,
    Add(String),
    NavigatePrevious,
    NavigateNext,
    ResetPosition,
}

pub struct HistoryEventHandler;

impl HistoryEventHandler {
    pub fn handle_command_result(result: &str) -> Option<HistoryEvent> {
        match result {
            "__CLEAR_HISTORY__" => Some(HistoryEvent::Clear),
            _ => None,
        }
    }

    pub fn create_clear_response() -> String {
        crate::i18n::get_translation("system.input.history_cleared", &[])
    }
}
