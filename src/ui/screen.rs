// ## FILE: ./src/screen.rs
// src/ui/screen.rs
use crate::core::prelude::*;
use crate::input::keyboard::{KeyAction, KeyboardManager};
use crate::output::logging::{AppLogger, LogMessage};

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

    pub async fn run(&mut self) -> Result<()> {
        if let Some(debug_info) = &self.config.debug_info {
            self.message_manager.add_message(debug_info.clone());
        }

        let result = loop {
            self.process_pending_logs().await;

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
                            KeyAction::Quit => {
                                self.events.shutdown().await;
                                break Ok(());
                            }
                            _ => {
                                if let Some(new_input) = self.input_state.handle_input(key) {
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
                            input_state.update_cursor_blink(); // Hier rufen wir direkt die Methode auf
                        }
                    }
                }
            }

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

            let available_height = chunks[0].height.saturating_sub(2) as usize;

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
