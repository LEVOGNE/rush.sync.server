// src/commands/lang/command.rs
use super::manager::LanguageManager;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct LanguageCommand;

impl LanguageCommand {
    pub fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("lang")
    }

    pub fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => Ok(LanguageManager::show_status()),
            Some(&lang) => match LanguageManager::switch_language_only(lang) {
                Ok(()) => {
                    let msg = crate::i18n::get_command_translation(
                        "system.commands.language.changed",
                        &[&lang.to_uppercase()],
                    );
                    Ok(LanguageManager::create_save_message_format(lang, &msg))
                }
                Err(e) => Ok(crate::i18n::get_command_translation(
                    "system.commands.language.invalid",
                    &[&e.to_string()],
                )),
            },
        }
    }

    pub async fn execute_async(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => Ok(LanguageManager::show_status()),
            Some(&lang) => LanguageManager::change_language(lang).await,
        }
    }

    pub fn supports_async(&self) -> bool {
        true // ✅ Language Command unterstützt async
    }
}
