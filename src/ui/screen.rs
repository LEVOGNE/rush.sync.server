// ## BEGIN ##
use crate::commands::history::HistoryKeyboardHandler;
use crate::commands::lang::LanguageManager;
use crate::core::prelude::*;
use crate::input::{
    event::{AppEvent, EventHandler},
    input::InputState,
    keyboard::{KeyAction, KeyboardManager},
};
use crate::output::{logging::AppLogger, message::MessageManager, output::create_output_widget};
use crate::ui::{color::AppColor, terminal::TerminalManager, widget::Widget};

use crossterm::event::KeyEvent;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, Stdout};

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

    /// ✅ Hauptloop: Nur Dispatcher, schlank & lesbar
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

    /// ✅ Eingaben (History, Submit, Scroll, Restart, Quit)
    async fn handle_input_event(&mut self, key: KeyEvent) -> Result<bool> {
        // History
        if HistoryKeyboardHandler::get_history_action(&key).is_some() {
            if let Some(new_input) = self.input_state.handle_input(key) {
                if let Some(processed) = LanguageManager::process_save_message(&new_input).await {
                    self.message_manager.add_message(processed);
                    return Ok(false);
                }
                self.message_manager.add_message(new_input.clone());

                if new_input.starts_with("__CLEAR__") {
                    self.message_manager.clear_messages();
                } else if new_input.starts_with("__EXIT__") {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // Normale Keys
        match self.keyboard_manager.get_action(&key) {
            KeyAction::ScrollUp
            | KeyAction::ScrollDown
            | KeyAction::PageUp
            | KeyAction::PageDown => {
                let window_height = self.get_content_height();
                self.message_manager
                    .handle_scroll(self.keyboard_manager.get_action(&key), window_height);
            }
            KeyAction::Submit => {
                if let Some(new_input) = self.input_state.handle_input(key) {
                    if let Some(processed) = LanguageManager::process_save_message(&new_input).await
                    {
                        self.message_manager.add_message(processed);
                        return Ok(false);
                    }

                    self.message_manager.add_message(new_input.clone());
                    if new_input.starts_with("__CLEAR__") {
                        self.message_manager.clear_messages();
                    } else if new_input.starts_with("__EXIT__") {
                        return Ok(true);
                    } else if new_input.starts_with("__RESTART_FORCE__")
                        || new_input == "__RESTART__"
                    {
                        if let Err(e) = self.perform_restart().await {
                            self.message_manager
                                .add_message(format!("Restart failed: {}", e));
                        }
                    }
                }
            }
            KeyAction::Quit => return Ok(true),
            _ => {
                if let Some(new_input) = self.input_state.handle_input(key) {
                    if let Some(processed) = LanguageManager::process_save_message(&new_input).await
                    {
                        self.message_manager.add_message(processed);
                        return Ok(false);
                    }
                    self.message_manager.add_message(new_input);
                }
            }
        }
        Ok(false)
    }

    /// ✅ Fenstergröße anpassen
    async fn handle_resize_event(&mut self, width: u16, height: u16) -> Result<()> {
        self.terminal_size = (width, height);
        let window_height = self.get_content_height();
        self.message_manager
            .scroll_state
            .update_dimensions(window_height, self.message_manager.get_content_height());
        Ok(())
    }

    /// ✅ Tick (Typewriter, Cursor-Blink)
    async fn handle_tick_event(&mut self) -> Result<()> {
        self.message_manager.update_typewriter();
        if let Some(input_state) = self.input_state.as_input_state() {
            input_state.update_cursor_blink();
        }
        Ok(())
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
                self.message_manager.add_message(
                    AppColor::from_any("error")
                        .format_message("ERROR", &format!("Logging-Fehler: {:?}", e)),
                );
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

    async fn perform_restart(&mut self) -> Result<()> {
        self.terminal_mgr.cleanup().await?;
        self.terminal_mgr = TerminalManager::new().await?;
        self.terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        self.terminal = Terminal::new(backend)?;

        self.message_manager.clear_messages();
        self.input_state = Box::new(InputState::new(&self.config.prompt.text, self.config));
        self.waiting_for_restart_confirmation = false;

        self.message_manager
            .add_message(crate::i18n::get_command_translation(
                "system.commands.restart.success",
                &[],
            ));
        log::info!("Internal restart completed successfully");
        Ok(())
    }
}
// ## END ##
