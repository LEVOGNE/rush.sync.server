// =====================================================
// FILE: src/ui/screen.rs - KORRIGIERTE CONSTRUCTOR CALLS
// =====================================================

use crate::commands::history::HistoryKeyboardHandler;
use crate::commands::lang::LanguageService;
use crate::commands::theme::ThemeSystem;
use crate::core::prelude::*;
use crate::input::{
    input::InputState,
    keyboard::{KeyAction, KeyboardManager},
};
use crate::input::{AppEvent, EventHandler};
use crate::output::{display::MessageDisplay, logging::AppLogger};
use crate::ui::{color::AppColor, terminal::TerminalManager, widget::Widget};

use crossterm::event::KeyEvent;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, Stdout};

pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

pub struct ScreenManager {
    terminal: TerminalBackend,
    message_display: MessageDisplay,
    input_state: Box<dyn Widget>,
    terminal_size: (u16, u16),
    config: Config,
    terminal_mgr: TerminalManager,
    events: EventHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_restart_confirmation: bool,
}

impl ScreenManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let mut terminal_mgr = TerminalManager::new().await?;
        terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let size = terminal.size()?;

        let initial_height = size.height.saturating_sub(4) as usize;
        let mut message_display = MessageDisplay::new(config);
        message_display
            .scroll_state
            .update_dimensions(initial_height, 0);

        let owned_config = config.clone();

        Ok(Self {
            terminal,
            terminal_mgr,
            message_display,
            // ✅ KORRIGIERT: InputState::new nimmt nur &Config
            input_state: Box::new(InputState::new(config)),
            terminal_size: (size.width, size.height),
            config: owned_config,
            events: EventHandler::new(config.poll_rate),
            keyboard_manager: KeyboardManager::new(),
            waiting_for_restart_confirmation: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let result = loop {
            if let Some(event) = self.events.next().await {
                match event {
                    AppEvent::Input(key) => {
                        if self.handle_input_event(key).await? {
                            self.events.shutdown().await;
                            break Ok(());
                        }
                    }
                    AppEvent::Resize(width, height) => {
                        self.handle_resize_event(width, height).await?;
                    }
                    AppEvent::Tick => {
                        self.handle_tick_event().await?;
                    }
                }
            }

            self.process_pending_logs().await;
            self.render().await?;
        };

        self.terminal_mgr.cleanup().await?;
        result
    }

    async fn handle_input_event(&mut self, key: KeyEvent) -> Result<bool> {
        if HistoryKeyboardHandler::get_history_action(&key).is_some() {
            if let Some(new_input) = self.input_state.handle_input(key) {
                if let Some(processed) = LanguageService::process_save_message(&new_input).await {
                    self.message_display.add_message(processed);
                    return Ok(false);
                }

                if let Some(processed) = self.process_live_theme_update(&new_input).await {
                    self.message_display.add_message(processed);
                    return Ok(false);
                }

                self.message_display.add_message(new_input.clone());

                if new_input.starts_with("__CLEAR__") {
                    self.message_display.clear_messages();
                } else if new_input.starts_with("__EXIT__") {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        match self.keyboard_manager.get_action(&key) {
            KeyAction::ScrollUp
            | KeyAction::ScrollDown
            | KeyAction::PageUp
            | KeyAction::PageDown => {
                let window_height = self.get_content_height();
                self.message_display
                    .handle_scroll(self.keyboard_manager.get_action(&key), window_height);
            }
            KeyAction::Submit => {
                if let Some(new_input) = self.input_state.handle_input(key) {
                    // ✅ DETECT Performance Commands BEFORE processing
                    let input_command = new_input.trim().to_lowercase();
                    let is_performance_command = input_command == "perf"
                        || input_command == "performance"
                        || input_command == "stats";

                    if let Some(processed) = LanguageService::process_save_message(&new_input).await
                    {
                        self.message_display.add_message(processed);
                        return Ok(false);
                    }

                    if let Some(processed) = self.process_live_theme_update(&new_input).await {
                        self.message_display.add_message(processed);
                        return Ok(false);
                    }

                    self.message_display.add_message(new_input.clone());

                    // ✅ SANFTER FIX für Performance Commands (ohne terminal.clear)
                    if is_performance_command {
                        log::info!("🔧 Performance command '{}' detected", input_command);

                        // Warten bis Message verarbeitet ist
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                        // Scroll-System sanft reparieren
                        self.message_display.scroll_state.force_auto_scroll();

                        let window_height = self.get_content_height();
                        let content_height = self.message_display.get_content_height();
                        self.message_display
                            .scroll_state
                            .update_dimensions(window_height, content_height);

                        log::info!("✅ Performance command: scroll reset applied");
                    }

                    // ✅ Standard Command-Verarbeitung
                    if new_input.starts_with("__CLEAR__") {
                        self.message_display.clear_messages();
                    } else if new_input.starts_with("__EXIT__") {
                        return Ok(true);
                    } else if new_input.starts_with("__RESTART_WITH_MSG__") {
                        let feedback_msg = new_input
                            .replace("__RESTART_WITH_MSG__", "")
                            .trim()
                            .to_string();

                        if !feedback_msg.is_empty() {
                            self.message_display.add_message(feedback_msg);
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }

                        if let Err(e) = self.perform_restart().await {
                            self.message_display
                                .add_message(format!("Restart failed: {}", e));
                        }
                    } else if new_input.starts_with("__RESTART_FORCE__")
                        || new_input == "__RESTART__"
                    {
                        if let Err(e) = self.perform_restart().await {
                            self.message_display
                                .add_message(format!("Restart failed: {}", e));
                        }
                    }
                }
            }
            KeyAction::Quit => return Ok(true),
            _ => {
                if let Some(new_input) = self.input_state.handle_input(key) {
                    if let Some(processed) = LanguageService::process_save_message(&new_input).await
                    {
                        self.message_display.add_message(processed);
                        return Ok(false);
                    }

                    if let Some(processed) = self.process_live_theme_update(&new_input).await {
                        self.message_display.add_message(processed);
                        return Ok(false);
                    }

                    self.message_display.add_message(new_input);
                }
            }
        }
        Ok(false)
    }

    async fn process_live_theme_update(&mut self, message: &str) -> Option<String> {
        if !message.starts_with("__LIVE_THEME_UPDATE__") {
            return None;
        }

        let parts: Vec<&str> = message.split("__MESSAGE__").collect();
        if parts.len() != 2 {
            log::error!("Invalid live theme update format: {}", message);
            return None;
        }

        let theme_part = parts[0].replace("__LIVE_THEME_UPDATE__", "");
        let display_message = parts[1];

        log::debug!("🎨 Processing live theme update: {}", theme_part);

        let theme_system = match ThemeSystem::load() {
            Ok(system) => system,
            Err(e) => {
                log::error!("Failed to load theme system: {}", e);
                return Some(format!("❌ Theme update failed: {}", e));
            }
        };

        if let Some(theme_def) = theme_system.get_theme(&theme_part) {
            match self.create_theme_from_definition(theme_def) {
                Ok(new_theme) => {
                    let backup = self.input_state.get_backup_data().unwrap_or_default();

                    self.config.theme = new_theme;
                    self.config.current_theme_name = theme_part.clone();

                    // ✅ KORRIGIERT: InputState::new nimmt nur &config
                    self.input_state = Box::new(InputState::new(&self.config));
                    self.input_state.restore_backup_data(backup.clone());

                    self.message_display.update_config(&self.config);

                    log::info!(
                        "✅ Live theme '{}' applied with prompt '{}' - {} history entries preserved!",
                        theme_part.to_uppercase(),
                        self.config.theme.prompt_text,
                        backup.history.len()
                    );
                    Some(display_message.to_string())
                }
                Err(e) => {
                    log::error!("❌ Failed to create theme: {}", e);
                    Some(format!("❌ Theme update failed: {}", e))
                }
            }
        } else {
            log::error!("❌ Theme '{}' not found in TOML", theme_part);
            Some(format!("❌ Theme '{}' not found in config", theme_part))
        }
    }

    fn create_theme_from_definition(
        &self,
        theme_def: &crate::commands::theme::ThemeDefinition,
    ) -> Result<crate::core::config::Theme> {
        use crate::ui::color::AppColor;

        Ok(crate::core::config::Theme {
            input_text: AppColor::from_string(&theme_def.input_text)?,
            input_bg: AppColor::from_string(&theme_def.input_bg)?,
            cursor: AppColor::from_string(&theme_def.cursor)?,
            output_text: AppColor::from_string(&theme_def.output_text)?,
            output_bg: AppColor::from_string(&theme_def.output_bg)?,
            prompt_text: theme_def.prompt_text.clone(),
            prompt_color: AppColor::from_string(&theme_def.prompt_color)?,
        })
    }

    async fn handle_resize_event(&mut self, width: u16, height: u16) -> Result<()> {
        eprintln!(
            "🔄 RESIZE EVENT: {}x{} → {}x{}",
            self.terminal_size.0, self.terminal_size.1, width, height
        );

        self.terminal_size = (width, height);
        let window_height = self.get_content_height();
        let content_height = self.message_display.get_content_height();

        self.message_display
            .scroll_state
            .update_dimensions(window_height, content_height);

        eprintln!(
            "   Window height: {}, Content height: {}",
            window_height, content_height
        );
        Ok(())
    }

    async fn handle_tick_event(&mut self) -> Result<()> {
        self.message_display.update_typewriter();
        if let Some(input_state) = self.input_state.as_input_state() {
            input_state.update_cursor_blink();
        }
        Ok(())
    }

    fn get_content_height(&self) -> usize {
        // Berechne verfügbare Höhe für Output-Bereich
        let total_height = self.terminal_size.1 as usize;
        let margin = 2; // top + bottom margin
        let input_area = 3; // Input braucht 3 Zeilen

        total_height
            .saturating_sub(margin)
            .saturating_sub(input_area)
    }

    async fn process_pending_logs(&mut self) {
        match AppLogger::get_messages() {
            Ok(messages) => {
                for log_msg in messages {
                    self.message_display.add_message(log_msg.formatted());
                }
            }
            Err(e) => {
                self.message_display.add_message(
                    AppColor::from_any("error")
                        .format_message("ERROR", &format!("Logging-Fehler: {:?}", e)),
                );
            }
        }
    }

    async fn render(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let size = frame.size();

            if size.width < 30 || size.height < 8 {
                return;
            }

            // ✅ KORRIGIERT: Berücksichtige margin in Berechnung
            let total_available = size.height.saturating_sub(2); // margin(1) = top+bottom = 2
            let input_needs = 3; // Input braucht minimal 3 Zeilen
            let output_gets = total_available.saturating_sub(input_needs);

            // ✅ SICHERE LAYOUT-CONSTRAINTS - mathematisch korrekt
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(output_gets), // ✅ Output: Berechnet
                    Constraint::Length(input_needs), // ✅ Input: Fest 3 Zeilen
                ])
                .split(size);

            let output_chunk = chunks[0];
            let input_chunk = chunks[1];

            // ✅ DEBUG: Prüfe Layout-Math
            let total_used = output_chunk.height + input_chunk.height + 2; // +2 für margin
            if total_used != size.height {
                // Verwende dein Log-System statt eprintln!
                log::warn!(
                    "⚠️ Layout-Math ERROR: terminal={}, used={}, output={}, input={}",
                    size.height,
                    total_used,
                    output_chunk.height,
                    input_chunk.height
                );
            }

            // ✅ UPDATE Scroll-Dimensionen mit korrekter Output-Höhe
            let total_lines = self.message_display.get_content_height();
            self.message_display
                .scroll_state
                .update_dimensions(output_chunk.height as usize, total_lines);

            // ✅ RENDER
            let (messages_data, config) = self
                .message_display
                .create_output_widget_for_rendering(output_chunk.height);
            let messages_refs: Vec<(&String, usize)> =
                messages_data.iter().map(|(s, l)| (s, *l)).collect();

            let output_widget = crate::output::display::create_output_widget(
                &messages_refs,
                output_chunk.height,
                &config,
            );
            frame.render_widget(output_widget, output_chunk);

            let input_widget = self.input_state.render();
            frame.render_widget(input_widget, input_chunk);
        })?;
        Ok(())
    }

    async fn perform_restart(&mut self) -> Result<()> {
        self.terminal_mgr.cleanup().await?;
        self.terminal_mgr = TerminalManager::new().await?;
        self.terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        self.terminal = Terminal::new(backend)?;

        self.message_display.clear_messages();
        // ✅ KORRIGIERT: InputState::new nimmt nur &config
        self.input_state = Box::new(InputState::new(&self.config));
        self.waiting_for_restart_confirmation = false;

        self.message_display
            .add_message(crate::i18n::get_command_translation(
                "system.commands.restart.success",
                &[],
            ));
        log::info!("Internal restart completed successfully");
        Ok(())
    }
}
