// src/commands/version/version.rs
use crate::core::constants::VERSION;
use crate::prelude::*;

pub struct VersionCommand;

impl VersionCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "version" | "ver")
    }

    pub fn execute(&self) -> Result<String> {
        let (msg, category) = get_translation_details("system.commands.version");
        let color = AppColor::from_category(category);
        Ok(color.format_message(&category.to_string(), &msg.replace("{}", VERSION)))
    }
}
