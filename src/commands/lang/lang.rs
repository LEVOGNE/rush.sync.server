// src/commands/lang/lang.rs
use crate::prelude::*;
use crate::setup::cfg_handler::ConfigHandler;
use crate::ui::color::{AppColor, ColorCategory}; // Explizit importieren, um Konflikte zu vermeiden

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
                let (current_text, _) = get_translation_details("system.commands.language.current");
                let (available_text, available_category) =
                    get_translation_details("system.commands.language.available");

                // Ersetze Platzhalter in den Übersetzungen
                let current = current_text.replace("{}", &current_lang);
                let available = available_text.replace("{}", &available_langs);

                // Hole die korrekten Farben
                let current_color = AppColor::from_category(ColorCategory::Language);
                let available_color = AppColor::from_category(available_category);

                // Formatiere die Nachrichten mit den entsprechenden Farben
                let colored_current = current_color.format_message("lang", &current);
                let colored_available =
                    available_color.format_message(&available_category.to_string(), &available);

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
