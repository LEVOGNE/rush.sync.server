use super::LanguageService;
use crate::commands::command::Command;
use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct LanguageCommand {
    service: std::sync::Mutex<LanguageService>,
}

impl LanguageCommand {
    pub fn new() -> Self {
        Self {
            service: std::sync::Mutex::new(LanguageService::new()),
        }
    }
}

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
        let service = match self.service.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::error!("Mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        match args.first() {
            None => Ok(service.show_status()),
            Some(&lang) => match service.switch_language_only(lang) {
                Ok(()) => {
                    let msg = crate::i18n::get_command_translation(
                        "system.commands.language.changed",
                        &[&lang.to_uppercase()],
                    );
                    Ok(service.create_save_message(lang, &msg))
                }
                Err(e) => Ok(crate::i18n::get_command_translation(
                    "system.commands.language.invalid",
                    &[&e.to_string()],
                )),
            },
        }
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let mut service = LanguageService::new();

            match args.first() {
                None => Ok(service.show_status()),
                Some(&lang) => service.change_language(lang).await,
            }
        })
    }

    fn supports_async(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        70
    }
}

impl Default for LanguageCommand {
    fn default() -> Self {
        Self::new()
    }
}
