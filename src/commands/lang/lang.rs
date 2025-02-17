use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;

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
        if self.config_handler.is_none() {
            self.config_handler = ConfigHandler::new().await.ok();
        }
    }

    pub fn execute(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => {
                let current_lang = get_current_language();
                let available_langs = get_available_languages().join(", ");

                // Hole Übersetzungsdetails für aktuelle Sprache
                let (_, lang_category) =
                    get_translation_details("system.commands.language.current");
                let (_, info_category) = get_translation_details("system.log.info");

                let lang_color = AppColor::from_category(lang_category);
                let info_color = AppColor::from_category(info_category);

                // Separate Formatierung für jede Nachricht
                let current = format!("[LANG] Aktuelle Sprache: {}", current_lang);
                let available = format!("[INFO] Verfügbare Sprachen: {}", available_langs);

                // Jede Nachricht mit eigenem ANSI-Code
                let colored_current =
                    format!("\x1B[{}m{}\x1B[0m", lang_color.to_ansi_code(), current);
                let colored_available =
                    format!("\x1B[{}m{}\x1B[0m", info_color.to_ansi_code(), available);

                // Mit Zeilenumbruch getrennt
                Ok(format!("{}\n{}", colored_current, colored_available))
            }

            Some(&lang) => match set_language(lang) {
                Ok(()) => {
                    let (msg, category) =
                        get_translation_details("system.commands.language.changed");
                    let formatted_msg = msg.replace("{}", &lang.to_uppercase());
                    Ok(AppColor::from_category(category)
                        .format_message(&category.to_string(), &formatted_msg))
                }
                Err(e) => {
                    let (_, category) = get_translation_details("system.commands.language.invalid");
                    Ok(AppColor::from_category(category)
                        .format_message(&category.to_string(), &e.to_string()))
                }
            },
        }
    }
}
