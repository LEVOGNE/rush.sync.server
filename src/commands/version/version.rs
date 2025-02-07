// src/commands/version/version.rs
use crate::core::constants::VERSION;
use crate::prelude::*;
use crate::ui::color::AppColor;

pub struct VersionCommand;

impl VersionCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "version" | "ver")
    }

    pub fn execute(&self) -> Result<String> {
        // Version in Cyan ausgeben wie bei LANG
        let color = AppColor::from_custom_level("VERSION");
        Ok(color.format_message("VERSION", &format!("Rush Sync v{}", VERSION)))
    }
}
