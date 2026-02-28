// src/ui/screen.rs
use crate::commands::{history::HistoryKeyboardHandler, lang::LanguageService, theme::ThemeSystem};
use crate::core::prelude::*;
use crate::input::{
    keyboard::{KeyAction, KeyboardManager},
    state::InputState,
    AppEvent, EventHandler,
};
use crate::output::display::MessageDisplay;
use crate::ui::{
    color::AppColor,
    terminal::TerminalManager,
    viewport::ScrollDirection,
    widget::{AnimatedWidget, CursorWidget, StatefulWidget, Widget},
};
use crossterm::{event::KeyEvent, execute};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    sync::OnceLock,
};

pub type TerminalBackend = Terminal<CrosstermBackend<Stdout>>;

pub struct ScreenManager {
    terminal: TerminalBackend,
    pub message_display: MessageDisplay,
    input_state: InputState,
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

        let mut screen_manager = Self {
            terminal,
            terminal_mgr,
            message_display: MessageDisplay::new(config, size.width, size.height),
            input_state: InputState::new(config),
            config: config.clone(),
            events: EventHandler::new(config.poll_rate),
            keyboard_manager: KeyboardManager::new(),
            waiting_for_restart_confirmation: false,
        };

        let version = crate::core::constants::VERSION;
        let startup_msg = get_command_translation("system.startup.version", &[version]);
        screen_manager
            .message_display
            .add_message_instant(startup_msg);

        Ok(screen_manager)
    }

    pub async fn run(&mut self) -> Result<()> {
        let result = loop {
            if let Some(event) = self.events.next().await {
                match event {
                    AppEvent::Input(key) => {
                        if self.handle_input(key).await? {
                            self.events.shutdown().await;
                            break Ok(());
                        }
                    }
                    AppEvent::Resize(w, h) => self.handle_resize(w, h).await?,
                    AppEvent::Tick => self.handle_tick().await?,
                }
            }
            self.render().await?;
        };
        self.terminal_mgr.cleanup().await?;
        result
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<bool> {
        // History handling
        if HistoryKeyboardHandler::get_history_action(&key).is_some() {
            if let Some(input) = self.input_state.handle_input(key) {
                self.process_special_input(&input).await;
            }
            return Ok(false);
        }

        // Scroll/Action handling
        match self.keyboard_manager.get_action(&key) {
            KeyAction::ScrollUp => {
                self.message_display.handle_scroll(ScrollDirection::Up, 1);
                Ok(false)
            }
            KeyAction::ScrollDown => {
                self.message_display.handle_scroll(ScrollDirection::Down, 1);
                Ok(false)
            }
            KeyAction::PageUp => {
                self.message_display
                    .handle_scroll(ScrollDirection::PageUp, 0);
                Ok(false)
            }
            KeyAction::PageDown => {
                self.message_display
                    .handle_scroll(ScrollDirection::PageDown, 0);
                Ok(false)
            }
            KeyAction::Submit => self.handle_submit(key).await,
            KeyAction::Quit => Ok(true),
            _ => {
                if let Some(input) = self.input_state.handle_input(key) {
                    self.process_special_input(&input).await;
                }
                Ok(false)
            }
        }
    }

    async fn handle_submit(&mut self, key: KeyEvent) -> Result<bool> {
        use crate::core::constants::*;
        let Some(input) = self.input_state.handle_input(key) else {
            return Ok(false);
        };

        if input == SIG_CLEAR {
            self.message_display.clear_messages();
            return Ok(false);
        }

        if input == SIG_EXIT {
            return Ok(true);
        }

        if input.starts_with(SIG_RESTART) {
            self.handle_restart(&input).await;
            return Ok(false);
        }

        // Process special messages (theme, language updates)
        if self.process_special_input(&input).await {
            return Ok(false);
        }

        // Only add to display if it was not a system command
        let cmd = input.trim().to_lowercase();
        if input.starts_with("__")
            || ["theme", "help", "lang"]
                .iter()
                .any(|&c| cmd.starts_with(c))
        {
            self.message_display.add_message_instant(input.clone());
        } else {
            self.message_display.add_message(input.clone());
        }

        Ok(false)
    }

    async fn process_special_input(&mut self, input: &str) -> bool {
        // Language updates
        if let Some(processed) = LanguageService::process_save_message(input).await {
            self.message_display.add_message_instant(processed);
            return true;
        }

        // Theme updates
        if let Some(processed) = self.process_theme_update(input).await {
            self.message_display.add_message_instant(processed);
            return true;
        }

        false
    }

    async fn process_theme_update(&mut self, message: &str) -> Option<String> {
        use crate::core::constants::*;
        if !message.starts_with(SIG_LIVE_THEME_UPDATE) {
            return None;
        }

        let parts: Vec<&str> = message.split(SIG_THEME_MSG_SEP).collect();
        if parts.len() != 2 {
            return None;
        }

        let theme_name = parts[0].replace(SIG_LIVE_THEME_UPDATE, "");
        let display_msg = parts[1];

        // Load and apply theme
        let theme_system = ThemeSystem::load().ok()?;
        let theme_def = theme_system.get_theme(&theme_name)?;
        let new_theme = self.create_theme(theme_def).ok()?;

        // Backup state, update config, restore state
        let backup = self.input_state.export_state();
        self.config.theme = new_theme;
        self.config.current_theme_name = theme_name;

        self.message_display.clear_messages();
        self.message_display.update_config(&self.config);

        self.input_state = InputState::new(&self.config);
        self.input_state.import_state(backup);

        Some(display_msg.to_string())
    }

    fn create_theme(
        &self,
        def: &crate::commands::theme::ThemeDefinition,
    ) -> Result<crate::core::config::Theme> {
        Ok(crate::core::config::Theme {
            input_text: AppColor::from_string(&def.input_text)?,
            input_bg: AppColor::from_string(&def.input_bg)?,
            output_text: AppColor::from_string(&def.output_text)?,
            output_bg: AppColor::from_string(&def.output_bg)?,
            input_cursor_prefix: def.input_cursor_prefix.clone(),
            input_cursor_color: AppColor::from_string(&def.input_cursor_color)?,
            input_cursor: def.input_cursor.clone(),
            output_cursor: def.output_cursor.clone(),
            output_cursor_color: AppColor::from_string(&def.output_cursor_color)?,
        })
    }

    async fn handle_restart(&mut self, input: &str) {
        use crate::core::constants::SIG_RESTART_WITH_MSG;
        if input.starts_with(SIG_RESTART_WITH_MSG) {
            let msg = input.replace(SIG_RESTART_WITH_MSG, "").trim().to_string();
            if !msg.is_empty() {
                self.message_display.add_message_instant(msg);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        if let Err(e) = self.perform_restart().await {
            self.message_display
                .add_message_instant(get_translation("screen.restart.failed", &[&e.to_string()]));
        }
    }

    async fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.message_display.handle_resize(width, height);
        Ok(())
    }

    async fn handle_tick(&mut self) -> Result<()> {
        self.message_display.update_typewriter();
        self.input_state.tick();
        Ok(())
    }

    async fn render(&mut self) -> Result<()> {
        let (input_widget, cursor_pos) = self.input_state.render_with_cursor();

        let viewport_ok = self.message_display.viewport().is_usable();
        let output_area = self.message_display.viewport().output_area();
        let input_area = self.message_display.viewport().input_area();

        let (messages, config, layout, cursor_state) =
            self.message_display.create_output_widget_for_rendering();

        self.terminal.draw(|frame| {
            let size = frame.size();

            // Emergency cases with i18n
            if size.width < 10 || size.height < 5 {
                let widget = ratatui::widgets::Paragraph::new(get_translation(
                    "screen.render.terminal_too_small",
                    &[],
                ))
                .block(ratatui::widgets::Block::default());
                frame.render_widget(widget, size);
                return;
            }

            if !viewport_ok || !output_area.is_valid() || !input_area.is_valid() {
                let widget = ratatui::widgets::Paragraph::new(get_translation(
                    "screen.render.viewport_error",
                    &[],
                ))
                .block(ratatui::widgets::Block::default());
                frame.render_widget(widget, size);
                return;
            }

            // Check bounds
            if Self::exceeds_bounds(&output_area, &input_area, size) {
                return;
            }

            // Render normally
            let output_widget = crate::output::display::create_output_widget(
                &messages,
                layout,
                &config,
                cursor_state,
            );

            frame.render_widget(output_widget, output_area.as_rect());
            frame.render_widget(input_widget, input_area.as_rect());

            if let Some((x, y)) = cursor_pos {
                frame.set_cursor(input_area.x + 3 + x, input_area.y + 1 + y);
            }
        })?;

        // Cursor styling (unchanged)
        if cursor_pos.is_some() {
            self.apply_cursor_styling()?;
        } else {
            execute!(std::io::stdout(), crossterm::style::Print("\x1B[?25l"))?;
        }
        Ok(())
    }

    fn exceeds_bounds(
        output: &crate::ui::viewport::LayoutArea,
        input: &crate::ui::viewport::LayoutArea,
        size: ratatui::layout::Rect,
    ) -> bool {
        output.x + output.width > size.width
            || output.y + output.height > size.height
            || input.x + input.width > size.width
            || input.y + input.height > size.height
    }

    fn apply_cursor_styling(&self) -> Result<()> {
        let form = match self.config.theme.input_cursor.to_uppercase().as_str() {
            "PIPE" => "\x1B[6 q",
            "UNDERSCORE" => "\x1B[4 q",
            "BLOCK" => "\x1B[2 q",
            _ => "\x1B[6 q",
        };

        let color_cmds = self.get_cursor_colors(&self.config.theme.input_cursor_color);

        execute!(std::io::stdout(), crossterm::style::Print(form))?;
        for cmd in color_cmds {
            execute!(std::io::stdout(), crossterm::style::Print(cmd))?;
        }
        execute!(std::io::stdout(), crossterm::style::Print("\x1B[?25h"))?;
        Ok(())
    }

    fn get_cursor_colors(&self, color: &AppColor) -> Vec<String> {
        let (r, g, b) = self.get_rgb(color);
        let info = Self::terminal_info();

        if info.tmux {
            return vec![format!(
                "\x1BPtmux;\x1B\x1B]12;#{:02x}{:02x}{:02x}\x07\x1B\\",
                r, g, b
            )];
        }

        let base = format!("\x1B]12;#{:02x}{:02x}{:02x}\x07", r, g, b);
        match info.term_program.as_str() {
            "Apple_Terminal" => vec![base],
            p if p.starts_with("iTerm") => {
                vec![format!("\x1B]Pl{:02x}{:02x}{:02x}\x1B\\", r, g, b), base]
            }
            _ => vec![base],
        }
    }

    fn get_rgb(&self, color: &AppColor) -> (u8, u8, u8) {
        match color.to_name() {
            "black" => (0, 0, 0),
            "red" => (255, 0, 0),
            "green" => (0, 255, 0),
            "yellow" => (255, 255, 0),
            "blue" => (0, 0, 255),
            "magenta" => (255, 0, 255),
            "cyan" => (0, 255, 255),
            "white" => (255, 255, 255),
            "gray" => (128, 128, 128),
            "darkgray" => (64, 64, 64),
            _ => (255, 255, 255),
        }
    }

    fn terminal_info() -> &'static TerminalInfo {
        TERMINAL_INFO.get_or_init(|| TerminalInfo {
            term_program: std::env::var("TERM_PROGRAM").unwrap_or_default(),
            tmux: std::env::var("TMUX").is_ok(),
        })
    }

    async fn perform_restart(&mut self) -> Result<()> {
        execute!(
            std::io::stdout(),
            crossterm::style::Print("\x1B[0 q"),
            crossterm::style::Print("\x1B[?25h")
        )?;

        self.terminal_mgr.cleanup().await?;
        self.terminal_mgr = TerminalManager::new().await?;
        self.terminal_mgr.setup().await?;

        let backend = CrosstermBackend::new(io::stdout());
        self.terminal = Terminal::new(backend)?;
        let size = self.terminal.size()?;

        self.message_display = MessageDisplay::new(&self.config, size.width, size.height);
        self.input_state = InputState::new(&self.config);
        self.waiting_for_restart_confirmation = false;

        self.message_display
            .add_message(get_translation("screen.restart.success", &[]));
        Ok(())
    }

    pub async fn switch_theme_safely(&mut self, theme_name: &str) -> Result<String> {
        let system = ThemeSystem::load().map_err(|e| {
            AppError::Validation(get_translation(
                "screen.theme.load_failed",
                &[&e.to_string()],
            ))
        })?;

        let def = system.get_theme(theme_name).ok_or_else(|| {
            AppError::Validation(get_translation("screen.theme.not_found", &[theme_name]))
        })?;

        let theme = self.create_theme(def)?;
        let backup = self.input_state.export_state();

        self.config.theme = theme;
        self.config.current_theme_name = theme_name.to_string();
        self.message_display.update_config(&self.config);

        self.input_state = InputState::new(&self.config);
        self.input_state.import_state(backup);

        Ok(get_translation(
            "screen.theme.switched_success",
            &[&theme_name.to_uppercase()],
        ))
    }

    /// Returns any missing i18n translation keys used by this module.
    pub fn validate_i18n_keys() -> Vec<String> {
        [
            "screen.theme.failed",
            "screen.render.too_small.text",
            "screen.render.viewport_error.text",
            "system.commands.restart.success",
        ]
        .iter()
        .filter(|&&key| !crate::i18n::has_translation(key))
        .map(|&key| key.to_string())
        .collect()
    }
}
