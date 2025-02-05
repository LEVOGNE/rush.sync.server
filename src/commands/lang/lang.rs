use crate::i18n;
use crate::prelude::*;
use crate::ui::color::AppColor;

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
                let current_lang = i18n::get_current_language();
                let available_langs = i18n::get_available_languages().join(", ");

                let current = AppColor::from_custom_level("LANG").format_message(
                    "LANG",
                    &i18n::get_translation("system.commands.language.current", &[&current_lang]),
                );

                let available = AppColor::from_custom_level("LANG").format_message(
                    "LANG",
                    &i18n::get_translation(
                        "system.commands.language.available",
                        &[&available_langs],
                    ),
                );

                Ok(format!("{}\n {}", current, available))
            }
            Some(&lang) => {
                match i18n::set_language(lang) {
                    Ok(()) => {
                        let msg = i18n::get_translation(
                            "system.commands.language.changed",
                            &[&lang.to_uppercase()],
                        );
                        Ok(AppColor::from_custom_level("LANG").format_message("LANG", &msg))
                    }
                    Err(e) => {
                        // Fehler weiterhin in Rot
                        Ok(AppColor::from_custom_level("ERROR")
                            .format_message("ERROR", &e.to_string()))
                    }
                }
            }
        }
    }
}
