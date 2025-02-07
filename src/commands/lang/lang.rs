use crate::i18n;
use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use crate::ui::color::AppColor;

pub struct LanguageCommand {
    config_handler: Option<ConfigHandler>,
}

impl LanguageCommand {
    pub fn new() -> Self {
        Self {
            config_handler: None,
        }
    }

    pub fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("lang")
    }

    pub async fn init(&mut self) {
        // Initialisiere den ConfigHandler nur wenn nötig
        if self.config_handler.is_none() {
            self.config_handler = ConfigHandler::new().await.ok();
        }
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
            Some(&lang) => match i18n::set_language(lang) {
                Ok(()) => {
                    let msg = i18n::get_translation(
                        "system.commands.language.changed",
                        &[&lang.to_uppercase()],
                    );

                    Ok(AppColor::from_custom_level("LANG").format_message("LANG", &msg))
                }
                Err(e) => Ok(
                    AppColor::from_custom_level("ERROR").format_message("ERROR", &e.to_string())
                ),
            },
        }
    }
}
