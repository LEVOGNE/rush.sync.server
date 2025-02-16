// src/commands/exit/exit.rs
use crate::prelude::*;
use crate::ui::color::AppColor;

pub struct ExitCommand;

impl ExitCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<String> {
        let (msg, category) = get_translation_details("system.input.confirm_exit");
        let color = AppColor::from_category(category);
        Ok(format!(
            "__CONFIRM_EXIT__{}",
            color.format_message(&category.to_string(), &msg) // Kategorie aus der JSON
        ))
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "exit" | "q")
    }
}
