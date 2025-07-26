use crate::core::prelude::*;
use crate::input::{
    event::{AppEvent, EventHandler},
    input::InputState,
    keyboard::{KeyAction, KeyboardManager},
};
use crate::output::{
    logging::{AppLogger, LogMessage},
    message::MessageManager,
    output::create_output_widget,
};
use crate::ui::{terminal::TerminalManager, widget::Widget};

use log::Level;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::Stdout;

// Type alias die vorher in prelude war
pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

pub struct ScreenManager<'a> {
    terminal: TerminalBackend,
    message_manager: MessageManager<'a>,
    input_state: Box<dyn Widget + 'a>,
    terminal_size: (u16, u16),
    config: &'a Config,
    terminal_mgr: TerminalManager,
    events: EventHandler,
}
impl<'a> ScreenManager<'a> {
    pub async fn new(config: &'a Config) -> Result<Self> {
        let mut terminal_mgr = TerminalManager::new().await?;
        terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let size = terminal.size()?;

        // Berechne initiale Fensterhöhe
        let initial_height = size.height.saturating_sub(4) as usize; // -4 für Margins und Input
        let mut message_manager = MessageManager::new(config);

        // Setze initiale Fensterhöhe
        message_manager
            .scroll_state
            .update_dimensions(initial_height, 0);

        Ok(Self {
            terminal,
            terminal_mgr,
            message_manager,
            input_state: Box::new(InputState::new(&config.prompt.text, config)),
            terminal_size: (size.width, size.height),
            config,
            events: EventHandler::new(config.poll_rate),
        })
    }

    // ✅ EINFACHE LÖSUNG: Nur diese Methode hinzufügen/ersetzen
    async fn handle_language_save(&mut self, message: &str) -> Option<String> {
        if message.starts_with("__SAVE_LANGUAGE__") {
            let parts: Vec<&str> = message.split("__MESSAGE__").collect();
            if parts.len() == 2 {
                let lang_part = parts[0].replace("__SAVE_LANGUAGE__", "");
                let display_message = parts[1];

                // ✅ DIREKTE SPRACH-AKTUALISIERUNG
                if let Err(e) = crate::i18n::set_language(&lang_part) {
                    return Some(format!("Fehler beim Setzen der Sprache: {}", e));
                }

                // ✅ KRITISCH: Cache leeren - das löst das Problem!
                crate::i18n::clear_translation_cache();

                // ✅ CONFIG SILENT SPEICHERN
                if let Err(e) = self.save_language_to_file_simple(&lang_part).await {
                    log::error!("Failed to save language config: {}", e);
                }

                return Some(display_message.to_string());
            }
        }
        None
    }

    // ✅ METHOD von ScreenManager
    async fn save_language_to_file_simple(&self, lang: &str) -> Result<()> {
        let config_paths = crate::setup::setup_toml::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;

                // ✅ SIMPLE LINE-BY-LINE UPDATE mit owned strings
                let lines: Vec<&str> = content.lines().collect();
                let mut new_lines: Vec<String> = Vec::new(); // ✅ Vec<String> statt Vec<&str>
                let mut in_language_section = false;
                let mut found_current = false;

                for line in lines {
                    if line.trim() == "[language]" {
                        in_language_section = true;
                        new_lines.push(line.to_string());
                    } else if line.starts_with('[')
                        && line.ends_with(']')
                        && line.trim() != "[language]"
                    {
                        in_language_section = false;
                        new_lines.push(line.to_string());
                    } else if in_language_section && line.trim().starts_with("current =") {
                        new_lines.push(format!("current = \"{}\"", lang)); // ✅ Owned string
                        found_current = true;
                    } else {
                        new_lines.push(line.to_string());
                    }
                }

                // Falls language section nicht existiert, hinzufügen
                if !found_current {
                    new_lines.push("".to_string());
                    new_lines.push("[language]".to_string());
                    new_lines.push(format!("current = \"{}\"", lang)); // ✅ Owned string
                }

                let new_content = new_lines.join("\n");
                tokio::fs::write(&path, new_content)
                    .await
                    .map_err(AppError::Io)?;

                log::debug!("Language '{}' saved to config", lang.to_uppercase());
                return Ok(());
            }
        }

        Ok(())
    }

    // ✅ BESTEHENDE run() METHODE ERWEITERN:
    pub async fn run(&mut self) -> Result<()> {
        let result = loop {
            if let Some(event) = self.events.next().await {
                match event {
                    AppEvent::Input(key) => {
                        let mut keyboard_manager = KeyboardManager::new();
                        match keyboard_manager.get_action(&key) {
                            action @ (KeyAction::ScrollUp
                            | KeyAction::ScrollDown
                            | KeyAction::PageUp
                            | KeyAction::PageDown) => {
                                let window_height = self.get_content_height();
                                self.message_manager.handle_scroll(action, window_height);
                            }
                            KeyAction::NoAction => {}
                            KeyAction::Submit => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
                                    // ✅ PRÜFE AUF LANGUAGE-SAVE MESSAGE
                                    if let Some(processed_message) =
                                        self.handle_language_save(&new_input).await
                                    {
                                        self.message_manager.add_message(processed_message);
                                        continue;
                                    }

                                    // ✅ NORMALE MESSAGE-VERARBEITUNG
                                    self.message_manager.add_message(new_input.clone());

                                    if new_input.starts_with("__CLEAR__") {
                                        self.message_manager.clear_messages();
                                        continue;
                                    } else if new_input.starts_with("__EXIT__") {
                                        self.events.shutdown().await;
                                        break Ok(());
                                    }
                                }
                            }
                            KeyAction::Quit => {
                                self.events.shutdown().await;
                                break Ok(());
                            }
                            _ => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
                                    // ✅ AUCH HIER PRÜFEN
                                    if let Some(processed_message) =
                                        self.handle_language_save(&new_input).await
                                    {
                                        self.message_manager.add_message(processed_message);
                                        continue;
                                    }
                                    self.message_manager.add_message(new_input);
                                }
                            }
                        }
                    }
                    AppEvent::Resize(width, height) => {
                        self.terminal_size = (width, height);
                        let window_height = self.get_content_height();
                        self.message_manager.scroll_state.update_dimensions(
                            window_height,
                            self.message_manager.get_content_height(),
                        );
                    }
                    AppEvent::Tick => {
                        self.message_manager.update_typewriter();
                        if let Some(input_state) = self.input_state.as_input_state() {
                            input_state.update_cursor_blink();
                        }
                    }
                }
            }

            self.process_pending_logs().await;
            self.render().await?;
        };

        self.terminal_mgr.cleanup().await?;
        result
    }

    // Neue Hilfsmethode zur Berechnung der verfügbaren Höhe
    fn get_content_height(&self) -> usize {
        self.terminal_size.1.saturating_sub(4) as usize // -4 für Margins und Input-Bereich
    }

    async fn process_pending_logs(&mut self) {
        match AppLogger::get_messages() {
            Ok(messages) => {
                for log_msg in messages {
                    self.message_manager.add_message(log_msg.formatted());
                }
            }
            Err(e) => {
                let error_msg = LogMessage::new(Level::Error, format!("Logging-Fehler: {:?}", e));
                self.message_manager.add_message(error_msg.formatted());
            }
        }
    }

    async fn render(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();

            // Prüfe minimale Größe
            if size.width < 20 || size.height < 10 {
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(size);

            let available_height = chunks[0].height as usize;

            // Aktualisiere ScrollState vor dem Rendering
            self.message_manager
                .scroll_state
                .update_dimensions(available_height, self.message_manager.get_content_height());

            let messages = self.message_manager.get_messages();
            let output_widget =
                create_output_widget(&messages, available_height as u16, self.config);

            frame.render_widget(output_widget, chunks[0]);

            let input_widget = self.input_state.render();
            frame.render_widget(input_widget, chunks[1]);
        })?;

        Ok(())
    }
}
