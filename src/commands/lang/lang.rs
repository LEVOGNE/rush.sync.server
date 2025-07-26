// commands/lang/lang.rs - ELEGANTE LÖSUNG OHNE ASYNC
use crate::core::prelude::*;
use crate::i18n::{
    get_available_languages, get_command_translation, get_current_language, set_language,
};

pub struct LanguageCommand;

impl LanguageCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("lang")
    }

    pub fn execute(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => {
                let current_lang = get_current_language();
                let available_langs = get_available_languages().join(", ");

                let current =
                    get_command_translation("system.commands.language.current", &[&current_lang]);

                let available = get_command_translation(
                    "system.commands.language.available",
                    &[&available_langs],
                );

                Ok(format!("{}\n{}", current, available))
            }
            Some(&lang) => {
                match set_language(lang) {
                    Ok(()) => {
                        // ✅ ELEGANTE LÖSUNG: Spezielle Message für Config-Update
                        Ok(format!(
                            "__SAVE_LANGUAGE__{}__MESSAGE__{}",
                            lang,
                            get_command_translation(
                                "system.commands.language.changed",
                                &[&lang.to_uppercase()],
                            )
                        ))
                    }
                    Err(e) => Ok(get_command_translation(
                        "system.commands.language.invalid",
                        &[&e.to_string()],
                    )),
                }
            }
        }
    }
}

impl Default for LanguageCommand {
    fn default() -> Self {
        Self::new()
    }
}
