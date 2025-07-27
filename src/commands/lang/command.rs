// =====================================================
// FILE: commands/lang/command.rs - ASYNC RICHTIG IMPLEMENTIERT
// =====================================================

use super::manager::LanguageManager;
use crate::commands::command::Command;
use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct LanguageCommand;

impl Command for LanguageCommand {
    fn name(&self) -> &'static str {
        "language"
    }

    fn description(&self) -> &'static str {
        "Change application language"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("lang")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
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

    /// ✅ ECHTES ASYNC - Override der Default-Implementierung
    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            match args.first() {
                None => Ok(LanguageManager::show_status()),
                Some(&lang) => LanguageManager::change_language(lang).await,
            }
        })
    }

    fn supports_async(&self) -> bool {
        true // ✅ Unterstützt echtes async
    }

    fn priority(&self) -> u8 {
        70 // Hohe Priorität für System-Commands
    }
}
