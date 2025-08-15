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
        let mut guard = self.theme_system.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
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
        // ✅ DEBUG: Zeige was passiert
        log::info!("🎨 ThemeCommand::execute_sync called with args: {:?}", args);

        // ✅ SICHERER THEME-SYSTEM LOAD mit Fehlerbehandlung
        let mut guard = match self.get_or_init_theme_system() {
            Ok(guard) => {
                log::info!("✅ ThemeSystem loaded successfully");
                guard
            }
            Err(e) => {
                log::error!("❌ ThemeSystem load failed: {}", e);
                return Ok(format!("❌ Theme system failed to load: {}\n\n💡 Tip: Add [theme.dark] section to rush.toml", e));
            }
        };

        let theme_system = guard.as_mut().unwrap();

        match args.first() {
            None => {
                log::info!("🎨 Calling theme_system.show_status()");
                let result = theme_system.show_status();
                log::info!("🎨 show_status result: '{}'", result);
                Ok(result)
            }
            Some(&"--help" | &"-h") => {
                log::info!("🎨 Calling create_help_text()");
                let result = Self::create_help_text(theme_system);
                log::info!("🎨 create_help_text result length: {} chars", result.len());
                // ✅ DEBUG: Zeige ersten Teil
                log::info!(
                    "🎨 create_help_text preview: '{}'",
                    &result[..result.len().min(100)]
                );
                Ok(result)
            }
            Some(&"debug") => match args.get(1) {
                Some(&theme_name) => Ok(theme_system.debug_theme_details(theme_name)),
                None => Ok("❌ Theme name missing. Usage: theme debug <name>".to_string()),
            },
            Some(&"preview") => match args.get(1) {
                Some(&theme_name) => theme_system.preview_theme(theme_name),
                None => Ok("❌ Theme name missing. Usage: theme preview <name>".to_string()),
            },
            Some(&theme_name) => {
                log::info!("🎨 Calling change_theme({})", theme_name);
                theme_system.change_theme(theme_name)
            }
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
            • output_cursor: BLOCK, PIPE, UNDERSCORE\n\
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
