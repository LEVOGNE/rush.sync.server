use crate::i18n::{
    get_available_languages, get_current_language, get_translation_details, set_language,
};

use crate::ui::color::{AppColor, ColorCategory};

pub struct LanguageCommand;

impl LanguageCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("lang")
    }

    pub fn execute(&self, args: &[&str]) -> crate::core::error::Result<String> {
        match args.first() {
            None => {
                let current_lang = get_current_language();
                let available_langs = get_available_languages().join(", ");
                let (current_text, _) = get_translation_details("system.commands.language.current");
                let (available_text, available_category) =
                    get_translation_details("system.commands.language.available");

                let current = current_text.replace("{}", &current_lang);
                let available = available_text.replace("{}", &available_langs);

                let current_color = AppColor::from_category(ColorCategory::Language);
                let available_color = AppColor::from_category(available_category);

                let colored_current = current_color.format_message("lang", &current);
                let colored_available =
                    available_color.format_message(&available_category.to_string(), &available);

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

impl Default for LanguageCommand {
    fn default() -> Self {
        Self::new()
    }
}
