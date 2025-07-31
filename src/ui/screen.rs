// =====================================================
// FILE: src/ui/screen.rs - MIT TERMINAL-CURSOR-SUPPORT
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
use crate::ui::{
    color::AppColor, terminal::TerminalManager, viewport::ScrollDirection, widget::Widget,
};

use crossterm::event::KeyEvent;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

use crossterm::execute;

pub struct ScreenManager {
    terminal: TerminalBackend,
    message_display: MessageDisplay,
    input_state: Box<dyn Widget>,
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

        let message_display = MessageDisplay::new(config, size.width, size.height);

        log::info!(
            "{}",
            t!(
                "screen.initialized",
                &size.width.to_string(),
                &size.height.to_string(),
                &message_display.viewport().debug_info()
            )
        );

        let owned_config = config.clone();

        Ok(Self {
            terminal,
            terminal_mgr,
            message_display,
            input_state: Box::new(InputState::new(config)),
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
        // HISTORY HANDLING ZUERST
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

        // SCROLL-HANDLING MIT VIEWPORT
        match self.keyboard_manager.get_action(&key) {
            KeyAction::ScrollUp => {
                self.message_display.handle_scroll(ScrollDirection::Up, 1);
                return Ok(false);
            }
            KeyAction::ScrollDown => {
                self.message_display.handle_scroll(ScrollDirection::Down, 1);
                return Ok(false);
            }
            KeyAction::PageUp => {
                self.message_display
                    .handle_scroll(ScrollDirection::PageUp, 0);
                return Ok(false);
            }
            KeyAction::PageDown => {
                self.message_display
                    .handle_scroll(ScrollDirection::PageDown, 0);
                return Ok(false);
            }
            KeyAction::Submit => {
                if let Some(new_input) = self.input_state.handle_input(key) {
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

                    if is_performance_command {
                        log::debug!(
                            "{}",
                            t!("screen.performance_command_detected", &input_command)
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        self.message_display.viewport_mut().force_auto_scroll();

                        log::debug!(
                            "{}",
                            t!("screen.performance_command_viewport_reset_applied")
                        );
                    }

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
            log::error!("{}", t!("screen.theme.invalid_format"));
            return None;
        }

        let theme_part = parts[0].replace("__LIVE_THEME_UPDATE__", "");
        let display_message = parts[1];

        log::info!(
            "🎨 LIVE THEME UPDATE STARTING: '{}' → '{}'",
            self.config.current_theme_name,
            theme_part
        );

        let theme_system = match ThemeSystem::load() {
            Ok(system) => system,
            Err(e) => {
                log::error!("{} {}", t!("screen.theme.load_failed"), e);
                return Some(tc!("screen.theme.failed", &e.to_string()));
            }
        };

        if let Some(theme_def) = theme_system.get_theme(&theme_part) {
            // ✅ DETAILED LOGGING: Jetzt mit theme_def im Scope
            log::info!(
                "🔍 THEME DETAILS: prefix: '{}' → '{}' | input_cursor: '{}' → '{}'",
                self.config.theme.input_cursor_prefix,
                theme_def.input_cursor_prefix,
                self.config.theme.input_cursor,
                theme_def.input_cursor
            );
            match self.create_theme_from_definition(theme_def) {
                Ok(new_theme) => {
                    let backup = self.input_state.get_backup_data().unwrap_or_default();

                    // ✅ DETAILED LOGGING: Show exact cursor transition
                    log::info!(
                        "🔄 THEME TRANSITION: old='{}'/prefix='{}'/input_cursor='{}'/output_cursor='{}' → new='{}'/prefix='{}'/input_cursor='{}'/output_cursor='{}'",
                        self.config.current_theme_name,
                        self.config.theme.input_cursor_prefix,
                        self.config.theme.input_cursor,
                        self.config.theme.output_cursor,
                        theme_part,
                        theme_def.input_cursor_prefix,
                        theme_def.input_cursor,
                        theme_def.output_cursor
                    );

                    // ✅ CRITICAL: Clear ALL UI state first
                    self.message_display.clear_messages();

                    // ✅ UPDATE CONFIG COMPLETELY
                    self.config.theme = new_theme;
                    self.config.current_theme_name = theme_part.clone();

                    // ✅ FORCE COMPLETE UI RECREATION mit zentraler Cursor-API
                    log::info!("🔄 FORCING MessageDisplay config update...");
                    self.message_display.update_config(&self.config);

                    log::info!("🔄 RECREATING InputState with central cursor API...");
                    self.input_state = Box::new(InputState::new(&self.config));
                    self.input_state.restore_backup_data(backup.clone());

                    // ✅ FINAL VERIFICATION
                    log::info!(
                        "✅ LIVE THEME APPLIED: theme='{}' | prefix='{}' | input_cursor='{}' | output_cursor='{}' | output_cursor_color='{}' | history={}",
                        theme_part.to_uppercase(),
                        self.config.theme.input_cursor_prefix,
                        self.config.theme.input_cursor,
                        self.config.theme.output_cursor,
                        self.config.theme.output_cursor_color.to_name(),
                        backup.history.len()
                    );

                    Some(display_message.to_string())
                }
                Err(e) => {
                    log::error!("{} {}", t!("screen.theme.load_failed"), e);
                    Some(tc!("screen.theme.failed", &e.to_string()))
                }
            }
        } else {
            log::error!("{} {}", t!("screen.theme.not_found"), theme_part);
            Some(tc!("screen.theme.not_found_feedback", theme_part.as_str()))
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

            // ✅ PERFEKTE CURSOR-KONFIGURATION
            input_cursor_prefix: theme_def.input_cursor_prefix.clone(),
            input_cursor_color: AppColor::from_string(&theme_def.input_cursor_color)?,
            input_cursor: theme_def.input_cursor.clone(),
            output_cursor: theme_def.output_cursor.clone(),
            output_cursor_color: AppColor::from_string(&theme_def.output_cursor_color)
                .unwrap_or_default(),
        })
    }

    async fn handle_resize_event(&mut self, width: u16, height: u16) -> Result<()> {
        log::info!(
            "{}",
            t!(
                "screen.resize_event",
                &self
                    .message_display
                    .viewport()
                    .terminal_size()
                    .0
                    .to_string(),
                &self
                    .message_display
                    .viewport()
                    .terminal_size()
                    .1
                    .to_string(),
                &width.to_string(),
                &height.to_string()
            )
        );

        let changed = self.message_display.handle_resize(width, height);

        if changed {
            log::info!(
                "{}",
                t!(
                    "screen.resize_completed",
                    &self.message_display.viewport().debug_info()
                )
            );
        }

        Ok(())
    }

    async fn handle_tick_event(&mut self) -> Result<()> {
        // ✅ TYPEWRITER-CURSOR UPDATE: Blinken + Progression
        self.message_display.update_typewriter();

        // ✅ INPUT-CURSOR UPDATE: Nur blinken (zentrale API)
        if let Some(input_state) = self.input_state.as_input_state() {
            input_state.update_cursor_blink();
        }
        Ok(())
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

    /// ✅ RENDER mit korrektem Cursor-Hide/Show
    async fn render(&mut self) -> Result<()> {
        // ✅ 1. CURSOR-INFO VOR draw() holen
        let (input_widget, cursor_pos) = self.input_state.render_with_cursor();

        self.terminal.draw(|frame| {
            let size = frame.size();

            if size.width < 10 || size.height < 5 {
                log::error!(
                    "{}",
                    t!(
                        "screen.render.too_small_log",
                        &size.width.to_string(),
                        &size.height.to_string()
                    )
                );

                let emergency_area = ratatui::layout::Rect {
                    x: 0,
                    y: 0,
                    width: size.width.max(1),
                    height: size.height.max(1),
                };

                let emergency_widget =
                    ratatui::widgets::Paragraph::new(t!("screen.render.too_small.text"))
                        .block(ratatui::widgets::Block::default());

                frame.render_widget(emergency_widget, emergency_area);
                return;
            }

            let viewport = self.message_display.viewport();

            if !viewport.is_usable() {
                log::error!("{}", t!("screen.render.viewport_not_usable_log"));

                let error_area = ratatui::layout::Rect {
                    x: 0,
                    y: 0,
                    width: size.width,
                    height: size.height,
                };

                let error_widget =
                    ratatui::widgets::Paragraph::new(t!("screen.render.viewport_error.text"))
                        .block(ratatui::widgets::Block::default());

                frame.render_widget(error_widget, error_area);
                return;
            }

            let output_area = viewport.output_area();
            let input_area = viewport.input_area();

            if !output_area.is_valid() || !input_area.is_valid() {
                log::error!(
                    "{}",
                    t!(
                        "screen.render.invalid_layout_log",
                        &output_area.width.to_string(),
                        &output_area.height.to_string(),
                        &output_area.x.to_string(),
                        &output_area.y.to_string(),
                        &input_area.width.to_string(),
                        &input_area.height.to_string(),
                        &input_area.x.to_string(),
                        &input_area.y.to_string()
                    )
                );
                return;
            }

            if output_area.x + output_area.width > size.width
                || output_area.y + output_area.height > size.height
                || input_area.x + input_area.width > size.width
                || input_area.y + input_area.height > size.height
            {
                log::error!(
                    "{}",
                    t!(
                        "screen.render.exceed_bounds_log",
                        &size.width.to_string(),
                        &size.height.to_string()
                    )
                );
                return;
            }

            // ✅ TYPEWRITER-CURSOR AWARE RENDERING
            let (visible_messages, config, output_layout, cursor_state) =
                self.message_display.create_output_widget_for_rendering();

            let message_refs: Vec<(String, usize, bool, bool, bool)> = visible_messages;

            let output_widget = crate::output::display::create_output_widget(
                &message_refs,
                output_layout,
                &config,
                cursor_state,
            );

            frame.render_widget(output_widget, output_area.as_rect());
            frame.render_widget(input_widget, input_area.as_rect());

            // ✅ CURSOR POSITION setzen (cursor_pos ist hier verfügbar durch Closure-Capture)
            if let Some((cursor_x, cursor_y)) = cursor_pos {
                let absolute_x = input_area.x + cursor_x;
                let absolute_y = input_area.y + cursor_y;
                frame.set_cursor(absolute_x, absolute_y);
            }
        })?;

        // ✅ 2. CURSOR-STIL setzen (NACH dem draw!)
        if cursor_pos.is_some() {
            // Cursor ist sichtbar → Cursor-Stil setzen
            let cursor_commands = self.get_terminal_cursor_commands();
            for command in cursor_commands {
                execute!(std::io::stdout(), crossterm::style::Print(command))?;
            }
        } else {
            // Cursor ist unsichtbar (Blinken) → Cursor verstecken
            execute!(
                std::io::stdout(),
                crossterm::style::Print("\x1B[?25l") // Hide cursor
            )?;
        }

        Ok(())
    }

    /// ✅ FIXED: Terminal-Cursor-Kommandos für korrekte Layer-Darstellung
    fn get_terminal_cursor_commands(&self) -> Vec<&'static str> {
        match self.config.theme.input_cursor.to_uppercase().as_str() {
            "PIPE" | "DEFAULT" => vec![
                "\x1B[6 q",  // Blinking bar (pipe)
                "\x1B[?25h", // Show cursor
            ],
            "UNDERSCORE" => vec![
                "\x1B[4 q",  // Blinking underscore
                "\x1B[?25h", // Show cursor
            ],
            "BLOCK" => vec![
                "\x1B[2 q",  // Blinking block (fallback, sollte nie erreicht werden)
                "\x1B[?25h", // Show cursor
            ],
            _ => vec![
                "\x1B[6 q",  // Default: Blinking bar
                "\x1B[?25h", // Show cursor
            ],
        }
    }

    async fn perform_restart(&mut self) -> Result<()> {
        log::info!("{}", t!("screen.restart.start"));

        self.terminal_mgr.cleanup().await?;
        self.terminal_mgr = TerminalManager::new().await?;
        self.terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        self.terminal = Terminal::new(backend)?;
        let size = self.terminal.size()?;

        self.message_display = MessageDisplay::new(&self.config, size.width, size.height);
        self.input_state = Box::new(InputState::new(&self.config));
        self.waiting_for_restart_confirmation = false;

        self.message_display
            .add_message(tc!("system.commands.restart.success"));

        log::info!("{}", t!("screen.restart.done"));
        Ok(())
    }
}

/// ✅ i18n Integration Helper (unverändert)
impl ScreenManager {
    pub fn validate_i18n_keys() -> Vec<String> {
        let required_keys = [
            "screen.performance_command_detected",
            "screen.performance_command_viewport_reset_applied",
            "screen.theme.invalid_format",
            "screen.theme.processing",
            "screen.theme.load_failed",
            "screen.theme.failed",
            "screen.theme.applied",
            "screen.theme.not_found",
            "screen.theme.not_found_feedback",
            "screen.render.too_small_log",
            "screen.render.too_small.text",
            "screen.render.viewport_not_usable_log",
            "screen.render.viewport_error.text",
            "screen.render.invalid_layout_log",
            "screen.render.exceed_bounds_log",
            "screen.restart.start",
            "screen.restart.done",
            "system.commands.restart.success",
        ];

        let mut missing = Vec::new();
        for key in required_keys {
            if !crate::i18n::has_translation(key) {
                missing.push(key.to_string());
            }
        }
        missing
    }

    pub fn get_i18n_debug_info() -> HashMap<String, usize> {
        let mut info = HashMap::new();
        let stats = crate::i18n::get_translation_stats();

        info.insert("screen_manager_keys".to_string(), 18);
        info.extend(stats);

        info
    }
}
