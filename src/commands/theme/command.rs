use super::ThemeSystem;
use crate::commands::command::Command;
use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug)]
pub struct ThemeCommand {
    theme_system: std::sync::Mutex<Option<ThemeSystem>>,
}

impl ThemeCommand {
    pub fn new() -> Self {
        Self {
            theme_system: std::sync::Mutex::new(None),
        }
    }

    fn get_or_init_theme_system(&self) -> Result<std::sync::MutexGuard<Option<ThemeSystem>>> {
        let mut guard = self.theme_system.lock().unwrap();
        if guard.is_none() {
            *guard = Some(ThemeSystem::load()?);
        }
        Ok(guard)
    }
}

impl Command for ThemeCommand {
    fn name(&self) -> &'static str {
        "theme"
    }

    fn description(&self) -> &'static str {
        "Change application theme (live update without restart, loaded from TOML)"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("theme")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let mut guard = self.get_or_init_theme_system()?;
        let theme_system = guard.as_mut().unwrap();

        match args.first() {
            None => Ok(theme_system.show_status()),
            Some(&"--help" | &"-h") => Ok(Self::create_help_text(theme_system)),
            Some(&"preview") => match args.get(1) {
                Some(&theme_name) => theme_system.preview_theme(theme_name),
                None => Ok("❌ Theme name missing. Usage: theme preview <name>".to_string()),
            },
            Some(&theme_name) => theme_system.change_theme(theme_name),
        }
    }

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move { self.execute_sync(args) })
    }

    fn supports_async(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        65
    }
}

impl ThemeCommand {
    fn create_help_text(theme_system: &ThemeSystem) -> String {
        let available_themes = theme_system.get_available_names();

        if available_themes.is_empty() {
            return "❌ Keine Themes verfügbar!\n\n📝 Füge [theme.xyz] Sektionen zur rush.toml hinzu:\n\n[theme.mein_theme]\ninput_text = \"White\"\ninput_bg = \"Black\"\ncursor = \"Green\"\noutput_text = \"Gray\"\noutput_bg = \"Black\"\nprompt_text = \">> \"\nprompt_color = \"Cyan\"\noutput_cursor = \"BLOCK\"    # ✅ NEU!\noutput_color = \"LightGreen\" # ✅ NEU!".to_string();
        }

        let themes_list = available_themes.join(", ");

        format!(
            "🎨 TOML-Theme Commands (Live Update - Geladen aus rush.toml!):\n\
            theme                Show available TOML-themes\n\
            theme <name>         Select theme: {}\n\
            theme preview <name> Preview theme colors + cursor config ✅ NEW!\n\
            theme -h             Show this help\n\n\
            ✨ Alle Themes werden LIVE aus [theme.*] Sektionen der rush.toml geladen!\n\
            🎯 NEU: Cursor-Konfiguration per output_cursor + output_color!\n\
            📁 Füge beliebige [theme.dein_name] Sektionen hinzu für neue Themes\n\
            🔄 Änderungen werden sofort angewendet (kein Restart nötig)\n\n\
            🎛️ Cursor-Optionen:\n\
            • output_cursor: DEFAULT, BLOCK, PIPE, UNDERSCORE\n\
            • output_color: Jede unterstützte Farbe (White, Green, etc.)",
            themes_list
        )
    }
}

impl Default for ThemeCommand {
    fn default() -> Self {
        Self::new()
    }
}
