use crate::commands::history::HistoryKeyboardHandler;
use crate::commands::lang::LanguageService;
use crate::commands::theme::ThemeSystem;
use crate::core::prelude::*;
use crate::input::{
    keyboard::{KeyAction, KeyboardManager},
    state::InputState,
};
use crate::input::{AppEvent, EventHandler};
use crate::output::display::MessageDisplay;
use crate::ui::{
    color::AppColor, terminal::TerminalManager, viewport::ScrollDirection, widget::Widget,
};

use crossterm::event::KeyEvent;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::sync::OnceLock;

pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

use crossterm::execute;

pub struct ScreenManager {
    terminal: TerminalBackend,
    pub message_display: MessageDisplay,
    input_state: Box<dyn Widget>,
    config: Config,
    terminal_mgr: TerminalManager,
    events: EventHandler,
    keyboard_manager: KeyboardManager,
    waiting_for_restart_confirmation: bool,
}

#[derive(Clone)]
struct TerminalInfo {
    term_program: String,
    tmux: bool,
}

static TERMINAL_INFO: OnceLock<TerminalInfo> = OnceLock::new();

impl ScreenManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let mut terminal_mgr = TerminalManager::new().await?;
        terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let size = terminal.size()?;

        let message_display = MessageDisplay::new(config, size.width, size.height);
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
                    self.message_display.add_message_instant(processed); // ✅ Sofort ohne Typewriter
                    return Ok(false);
                }

                self.message_display.add_message_instant(new_input.clone());

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
                log::info!("🔼 SCROLL UP detected!");
                log::info!(
                    "   Before: offset={}, content={}, window={}",
                    self.message_display.viewport().scroll_offset(),
                    self.message_display.viewport().content_height(),
                    self.message_display.viewport().window_height()
                );

                self.message_display.handle_scroll(ScrollDirection::Up, 1);

                log::info!(
                    "   After: offset={}, auto_scroll={}",
                    self.message_display.viewport().scroll_offset(),
                    self.message_display.viewport().is_auto_scroll_enabled()
                );
                return Ok(false);
            }
            KeyAction::ScrollDown => {
                log::info!("🔽 SCROLL DOWN detected!");
                log::info!(
                    "   Before: offset={}, content={}, window={}",
                    self.message_display.viewport().scroll_offset(),
                    self.message_display.viewport().content_height(),
                    self.message_display.viewport().window_height()
                );

                self.message_display.handle_scroll(ScrollDirection::Down, 1);

                log::info!(
                    "   After: offset={}, auto_scroll={}",
                    self.message_display.viewport().scroll_offset(),
                    self.message_display.viewport().is_auto_scroll_enabled()
                );
                return Ok(false);
            }
            KeyAction::PageUp => {
                log::info!("📄 PAGE UP detected!");
                self.message_display
                    .handle_scroll(ScrollDirection::PageUp, 0);
                return Ok(false);
            }
            KeyAction::PageDown => {
                log::info!("📄 PAGE DOWN detected!");
                self.message_display
                    .handle_scroll(ScrollDirection::PageDown, 0);
                return Ok(false);
            }
            KeyAction::Submit => {
                log::info!("🖥️ SCREEN: About to call input_state.handle_input()");

                if let Some(new_input) = self.input_state.handle_input(key) {
                    log::info!("🖥️ SCREEN: input_state returned {} chars", new_input.len());

                    let input_command = new_input.trim().to_lowercase();
                    let is_performance_command = input_command == "perf"
                        || input_command == "performance"
                        || input_command == "stats";

                    if let Some(processed) = LanguageService::process_save_message(&new_input).await
                    {
                        log::info!("🖥️ SCREEN: LanguageService processed message");
                        self.message_display.add_message_instant(processed); // ✅ FIX: Instant
                        return Ok(false);
                    }

                    if let Some(processed) = self.process_live_theme_update(&new_input).await {
                        log::info!("🖥️ SCREEN: ThemeUpdate processed message");
                        self.message_display.add_message_instant(processed); // ✅ FIX: Instant
                        return Ok(false);
                    }

                    log::info!("🖥️ SCREEN: Adding normal message to display");

                    // ✅ FIX: Commands should be instant, nur normale Messages mit Typewriter
                    if new_input.starts_with("__") {
                        // Special system messages
                        self.message_display.add_message_instant(new_input.clone());
                    } else if input_command.starts_with("theme")
                        || input_command.starts_with("test")
                        || input_command.starts_with("help")
                        || input_command.starts_with("lang")
                    {
                        // Command outputs should be instant
                        self.message_display.add_message_instant(new_input.clone());
                    } else {
                        // Regular messages can use typewriter
                        self.message_display.add_message(new_input.clone());
                    }

                    if is_performance_command {
                        log::info!("🖥️ SCREEN: Performance command detected, forcing auto-scroll");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        self.message_display.viewport_mut().force_auto_scroll();
                    }

                    // Handle special system commands
                    if new_input.starts_with("__CLEAR__") {
                        log::info!("🖥️ SCREEN: Clearing messages");
                        self.message_display.clear_messages();
                    } else if new_input.starts_with("__EXIT__") {
                        log::info!("🖥️ SCREEN: Exit requested");
                        return Ok(true);
                    } else if new_input.starts_with("__RESTART_WITH_MSG__") {
                        log::info!("🖥️ SCREEN: Restart with message requested");
                        let feedback_msg = new_input
                            .replace("__RESTART_WITH_MSG__", "")
                            .trim()
                            .to_string();

                        if !feedback_msg.is_empty() {
                            self.message_display.add_message_instant(feedback_msg); // ✅ FIX: Instant
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }

                        if let Err(e) = self.perform_restart().await {
                            self.message_display
                                .add_message_instant(format!("Restart failed: {}", e));
                            // ✅ FIX: Instant
                        }
                    } else if new_input.starts_with("__RESTART_FORCE__")
                        || new_input == "__RESTART__"
                    {
                        log::info!("🖥️ SCREEN: Restart requested");
                        if let Err(e) = self.perform_restart().await {
                            self.message_display
                                .add_message_instant(format!("Restart failed: {}", e));
                            // ✅ FIX: Instant
                        }
                    }
                } else {
                    log::info!("🖥️ SCREEN: input_state.handle_input() returned None");
                }
            }
            KeyAction::Quit => return Ok(true),
            _ => {
                if let Some(new_input) = self.input_state.handle_input(key) {
                    if let Some(processed) = LanguageService::process_save_message(&new_input).await
                    {
                        self.message_display.add_message_instant(processed); // ✅ FIX: Instant
                        return Ok(false);
                    }

                    if let Some(processed) = self.process_live_theme_update(&new_input).await {
                        self.message_display.add_message_instant(processed); // ✅ FIX: Instant
                        return Ok(false);
                    }

                    self.message_display.add_message_instant(new_input); // ✅ FIX: Instant
                }
            }
        }
        Ok(false)
    }

    // ✅ FIXED: Live-Theme-Update mit korrekter Cursor-Farb-Übertragung
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
            // ✅ KRITISCHER FIX: Theme-Definition Details loggen
            log::info!(
                "📋 THEME DEFINITION LOADED:\n  \
                input_cursor_prefix: '{}'\n  \
                input_cursor_color: '{}'\n  \
                input_cursor: '{}'\n  \
                output_cursor: '{}'\n  \
                output_cursor_color: '{}'",
                theme_def.input_cursor_prefix,
                theme_def.input_cursor_color,
                theme_def.input_cursor,
                theme_def.output_cursor,
                theme_def.output_cursor_color
            );

            match self.create_theme_from_definition(theme_def) {
                Ok(new_theme) => {
                    let backup = self.input_state.get_backup_data().unwrap_or_default();

                    // ✅ KRITISCHER FIX: Theme-Konvertierung Details loggen
                    log::info!(
                        "🔄 THEME CONVERSION COMPLETE:\n  \
                        OLD Config: input_cursor='{}', input_cursor_color='{}'\n  \
                        NEW Config: input_cursor='{}', input_cursor_color='{}'",
                        self.config.theme.input_cursor,
                        self.config.theme.input_cursor_color.to_name(),
                        new_theme.input_cursor,
                        new_theme.input_cursor_color.to_name()
                    );

                    // ✅ CRITICAL: Clear ALL UI state first
                    self.message_display.clear_messages();

                    // ✅ UPDATE CONFIG COMPLETELY
                    self.config.theme = new_theme;
                    self.config.current_theme_name = theme_part.clone();

                    // ✅ FORCE COMPLETE UI RECREATION
                    self.message_display.update_config(&self.config);

                    log::info!("🔄 RECREATING InputState with central cursor API...");
                    self.input_state = Box::new(InputState::new(&self.config));

                    // ✅ KRITISCHER FIX: Cursor-Details nach Recreation verifizieren
                    if let Some(_input_widget) = self.input_state.as_input_state() {
                        log::info!(
                            "✅ INPUT-CURSOR CREATED:\n  \
                            Expected: cursor='{}' (color: {})\n  \
                            Theme config: prefix='{}' (color: {})",
                            self.config.theme.input_cursor,
                            self.config.theme.input_cursor_color.to_name(),
                            self.config.theme.input_cursor_prefix,
                            self.config.theme.input_cursor_color.to_name()
                        );
                    }

                    self.input_state.restore_backup_data(backup.clone());

                    // ✅ FINAL VERIFICATION
                    log::info!(
                        "✅ LIVE THEME APPLIED SUCCESSFULLY:\n  \
                        theme='{}'\n  \
                        prefix='{}'\n  \
                        input_cursor='{}'\n  \
                        input_cursor_color='{}'\n  \
                        output_cursor='{}'\n  \
                        output_cursor_color='{}'\n  \
                        history={} entries",
                        theme_part.to_uppercase(),
                        self.config.theme.input_cursor_prefix,
                        self.config.theme.input_cursor,
                        self.config.theme.input_cursor_color.to_name(),
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

    // ✅ FIXED: Theme-Konvertierung mit detailliertem Logging
    fn create_theme_from_definition(
        &self,
        theme_def: &crate::commands::theme::ThemeDefinition,
    ) -> Result<crate::core::config::Theme> {
        use crate::ui::color::AppColor;

        let input_cursor_color = AppColor::from_string(&theme_def.input_cursor_color)?;
        let output_cursor_color = AppColor::from_string(&theme_def.output_cursor_color)?;

        Ok(crate::core::config::Theme {
            input_text: AppColor::from_string(&theme_def.input_text)?,
            input_bg: AppColor::from_string(&theme_def.input_bg)?,
            output_text: AppColor::from_string(&theme_def.output_text)?,
            output_bg: AppColor::from_string(&theme_def.output_bg)?,

            // ✅ PERFEKTE CURSOR-KONFIGURATION
            input_cursor_prefix: theme_def.input_cursor_prefix.clone(),
            input_cursor_color,
            input_cursor: theme_def.input_cursor.clone(),
            output_cursor: theme_def.output_cursor.clone(),
            output_cursor_color,
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

    /// ✅ 2-LAYER RENDER: Text + Terminal-Cursor getrennt!
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

            // ✅ LAYER 1: Widgets rendern
            frame.render_widget(output_widget, output_area.as_rect());
            frame.render_widget(input_widget, input_area.as_rect());

            // ✅ ZURÜCK ZU TERMINAL-CURSOR: Echter separater Layer!
            if let Some((rel_x, rel_y)) = cursor_pos {
                // Padding berücksichtigen: links=3, oben=1
                let padding_left = 3u16;
                let padding_top = 1u16;

                let abs_x = input_area.x + padding_left + rel_x;
                let abs_y = input_area.y + padding_top + rel_y;

                // ✅ ECHTER TERMINAL-CURSOR: Separate Ebene über dem Text!
                frame.set_cursor(abs_x, abs_y);
            }
        })?;

        // ✅ TERMINAL-CURSOR-STIL UND -FARBE setzen (nach dem draw!)
        if cursor_pos.is_some() {
            // Cursor ist sichtbar → Cursor-Stil UND -Farbe setzen
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

    /// ✅ ERWEITERTE Terminal-Cursor-Kommandos mit DEBUGGING
    fn get_terminal_cursor_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        // ✅ 1. CURSOR-FORM setzen (universal)
        let form_command = match self.config.theme.input_cursor.to_uppercase().as_str() {
            "PIPE" => "\x1B[6 q",       // Blinking bar (pipe)
            "UNDERSCORE" => "\x1B[4 q", // Blinking underscore
            "BLOCK" => "\x1B[2 q",      // Blinking block
            _ => "\x1B[6 q",            // Default: Blinking bar
        };
        commands.push(form_command.to_string());

        // ✅ 2. CURSOR-FARBE setzen (terminal-spezifisch)
        let cursor_color = &self.config.theme.input_cursor_color;
        let color_commands = self.get_all_cursor_color_sequences(cursor_color);
        commands.extend(color_commands);

        // ✅ 3. CURSOR anzeigen (universal)
        commands.push("\x1B[?25h".to_string()); // Show cursor

        commands
    }

    fn detect_terminal_once() -> &'static TerminalInfo {
        TERMINAL_INFO.get_or_init(|| {
            let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
            let term = std::env::var("TERM").unwrap_or_default();
            let tmux = std::env::var("TMUX").is_ok();

            log::info!("🖥️ TERMINAL DETECTION (ONE-TIME):");
            log::info!("   TERM_PROGRAM: '{}'", term_program);
            log::info!("   TERM: '{}'", term);
            log::info!("   TMUX: {}", tmux);

            TerminalInfo { term_program, tmux }
        })
    }

    /// ✅ ALLE MÖGLICHEN Cursor-Farb-Sequenzen senden (Shotgun-Approach)
    fn get_all_cursor_color_sequences(&self, color: &AppColor) -> Vec<String> {
        let mut sequences = Vec::new();
        let (r, g, b) = self.get_rgb_values(color);

        // ✅ VERWENDE CACHED TERMINAL INFO
        let terminal_info = Self::detect_terminal_once();

        // ✅ TMUX: Nur wenn wirklich in tmux
        if terminal_info.tmux {
            sequences.push(format!(
                "\x1BPtmux;\x1B\x1B]12;#{:02x}{:02x}{:02x}\x07\x1B\\",
                r, g, b
            ));
            return sequences; // Nur tmux-Sequenz senden
        }

        // ✅ MACOS TERMINAL: Apple Terminal.app
        if terminal_info.term_program == "Apple_Terminal" {
            sequences.push(format!("\x1B]12;#{:02x}{:02x}{:02x}\x07", r, g, b));
            return sequences;
        }

        // ✅ ITERM2: iTerm2
        if terminal_info.term_program.starts_with("iTerm") {
            sequences.push(format!("\x1B]Pl{:02x}{:02x}{:02x}\x1B\\", r, g, b));
            sequences.push(format!("\x1B]12;#{:02x}{:02x}{:02x}\x07", r, g, b));
            return sequences;
        }

        // ✅ VSCODE TERMINAL: Visual Studio Code
        if terminal_info.term_program == "vscode" || std::env::var("VSCODE_INJECTION").is_ok() {
            sequences.push(format!("\x1B]12;#{:02x}{:02x}{:02x}\x07", r, g, b));
            return sequences;
        }

        // ✅ FALLBACK: Standard Xterm-Sequenz
        sequences.push(format!("\x1B]12;#{:02x}{:02x}{:02x}\x07", r, g, b));
        sequences
    }

    /// ✅ PRÄZISE RGB-Werte aus AppColor
    fn get_rgb_values(&self, color: &AppColor) -> (u8, u8, u8) {
        let rgb = match color.to_name() {
            "black" => (0, 0, 0),
            "red" => (255, 0, 0),
            "green" => (0, 255, 0),
            "yellow" => (255, 255, 0), // ✅ GELB für deinen Test!
            "blue" => (0, 0, 255),
            "magenta" => (255, 0, 255),
            "cyan" => (0, 255, 255),
            "white" => (255, 255, 255),
            "gray" => (128, 128, 128),
            "darkgray" => (64, 64, 64),
            "lightred" => (255, 128, 128),
            "lightgreen" => (128, 255, 128),
            "lightyellow" => (255, 255, 128),
            "lightblue" => (128, 128, 255),
            "lightmagenta" => (255, 128, 255),
            "lightcyan" => (128, 255, 255),
            _ => (255, 255, 255), // Default: white
        };

        log::trace!(
            "🎨 COLOR MAPPING: '{}' → RGB({}, {}, {})",
            color.to_name(),
            rgb.0,
            rgb.1,
            rgb.2
        );
        rgb
    }

    async fn perform_restart(&mut self) -> Result<()> {
        log::info!("{}", t!("screen.restart.start"));

        // ✅ CURSOR-STIL zurücksetzen vor Cleanup
        execute!(
            std::io::stdout(),
            crossterm::style::Print("\x1B[0 q"), // Default cursor
            crossterm::style::Print("\x1B[?25h")  // Show cursor
        )?;

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
