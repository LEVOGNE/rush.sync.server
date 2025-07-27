// =====================================================
// FILE: ui/screen.rs - FINAL VERSION (ohne Debug)
// =====================================================

use crate::commands::history::HistoryKeyboardHandler;
use crate::commands::lang::LanguageManager;
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

pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

pub struct ScreenManager<'a> {
    terminal: TerminalBackend,
    message_manager: MessageManager<'a>,
    input_state: Box<dyn Widget + 'a>,
    terminal_size: (u16, u16),
    config: &'a Config,
    terminal_mgr: TerminalManager,
    events: EventHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_restart_confirmation: bool,
}

impl<'a> ScreenManager<'a> {
    pub async fn new(config: &'a Config) -> Result<Self> {
        let mut terminal_mgr = TerminalManager::new().await?;
        terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let size = terminal.size()?;

        let initial_height = size.height.saturating_sub(4) as usize;
        let mut message_manager = MessageManager::new(config);

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
            keyboard_manager: KeyboardManager::new(),
            waiting_for_restart_confirmation: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let result = loop {
            if let Some(event) = self.events.next().await {
                match event {
                    AppEvent::Input(key) => {
                        // ✅ 1. ZUERST: Prüfe ob es History-Keys sind (Up/Down ohne Modifier)
                        if let Some(_history_action) =
                            HistoryKeyboardHandler::get_history_action(&key)
                        {
                            // ✅ DIREKT an input_state weiterleiten
                            if let Some(new_input) = self.input_state.handle_input(key) {
                                if let Some(processed_message) =
                                    LanguageManager::process_save_message(&new_input).await
                                {
                                    self.message_manager.add_message(processed_message);
                                    continue;
                                }

                                self.message_manager.add_message(new_input.clone());

                                if new_input.starts_with("__CLEAR__") {
                                    self.message_manager.clear_messages();
                                    continue;
                                } else if new_input.starts_with("__EXIT__") {
                                    self.events.shutdown().await;
                                    break Ok(());
                                }
                            }
                            continue; // ✅ WICHTIG: Keine weitere Verarbeitung!
                        }

                        // ✅ 2. NORMAL: Andere Keys normal verarbeiten
                        //let mut keyboard_manager = KeyboardManager::new();
                        match self.keyboard_manager.get_action(&key) {
                            action @ (KeyAction::ScrollUp
                            | KeyAction::ScrollDown
                            | KeyAction::PageUp
                            | KeyAction::PageDown) => {
                                let window_height = self.get_content_height();
                                self.message_manager.handle_scroll(action, window_height);
                            }
                            KeyAction::NoAction => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
                                    if let Some(processed_message) =
                                        LanguageManager::process_save_message(&new_input).await
                                    {
                                        self.message_manager.add_message(processed_message);
                                        continue;
                                    }
                                    self.message_manager.add_message(new_input);
                                }
                            }
                            KeyAction::Submit => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
                                    if let Some(processed_message) =
                                        LanguageManager::process_save_message(&new_input).await
                                    {
                                        self.message_manager.add_message(processed_message);
                                        continue;
                                    }

                                    self.message_manager.add_message(new_input.clone());

                                    if new_input.starts_with("__CLEAR__") {
                                        self.message_manager.clear_messages();
                                        continue;
                                    } else if new_input.starts_with("__EXIT__") {
                                        self.events.shutdown().await;
                                        break Ok(());
                                    } else if new_input.starts_with("__RESTART_FORCE__")
                                        || new_input == "__RESTART__"
                                    {
                                        // ✅ KALT-RESTART durchführen
                                        if let Err(e) = self.perform_restart().await {
                                            self.message_manager
                                                .add_message(format!("Restart failed: {}", e));
                                        }
                                        continue;
                                    }
                                }
                            }
                            KeyAction::Quit => {
                                self.events.shutdown().await;
                                break Ok(());
                            }
                            _ => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
                                    if let Some(processed_message) =
                                        LanguageManager::process_save_message(&new_input).await
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

    fn get_content_height(&self) -> usize {
        self.terminal_size.1.saturating_sub(4) as usize
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

            if size.width < 20 || size.height < 10 {
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(size);

            let available_height = chunks[0].height as usize;

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

    /// Führt einen internen Kalt-Restart durch
    async fn perform_restart(&mut self) -> Result<()> {
        // ✅ 1. CLEANUP: Terminal zurücksetzen
        self.terminal_mgr.cleanup().await?;

        // ✅ 2. REINIT: Alles neu initialisieren
        self.terminal_mgr = TerminalManager::new().await?;
        self.terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        self.terminal = Terminal::new(backend)?;

        // ✅ 3. RESET: Messages und Input State zurücksetzen
        self.message_manager.clear_messages();
        self.input_state = Box::new(InputState::new(&self.config.prompt.text, self.config));
        self.waiting_for_restart_confirmation = false;

        // ✅ 4. SUCCESS: Restart-Nachricht
        self.message_manager
            .add_message(crate::i18n::get_command_translation(
                "system.commands.restart.success",
                &[],
            ));

        log::info!("Internal restart completed successfully");
        Ok(())
    }
}
