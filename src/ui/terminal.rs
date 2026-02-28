use crate::core::constants::APP_TITLE;
use crate::core::prelude::*;
use crate::i18n::get_translation;
use crossterm::{
    cursor, execute,
    style::ResetColor,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::Stdout;

pub struct TerminalManager {
    stdout: Stdout,
    raw_mode_enabled: bool,
}

impl TerminalManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            stdout: io::stdout(),
            raw_mode_enabled: false,
        })
    }

    pub async fn setup(&mut self) -> Result<()> {
        self.enable_full_raw_mode().await?;
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            EnterAlternateScreen,
            terminal::DisableLineWrap,
            terminal::SetTitle(APP_TITLE),
            crossterm::style::Print("\x1B]112\x07"),
            crossterm::style::Print("\x1B[0 q"),
            cursor::Hide
        )?;
        Ok(())
    }

    async fn enable_full_raw_mode(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.raw_mode_enabled = true;
        execute!(
            self.stdout,
            crossterm::style::Print("\x1B[?1000h"),
            crossterm::style::Print("\x1B[?1002h"),
            crossterm::style::Print("\x1B[?1015h"),
            crossterm::style::Print("\x1B[?1006h"),
            crossterm::style::Print("\x1B[?1049h")
        )?;
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        log::info!("Starting terminal cleanup...");

        if self.raw_mode_enabled {
            self.disable_full_raw_mode().await?;
        }

        // Multi-standard cursor reset for maximum compatibility
        execute!(
            self.stdout,
            crossterm::style::Print("\x1B]12;\x07"),
            crossterm::style::Print("\x1B]Pl\x1B\\"),
            crossterm::style::Print("\x1B]112\x07"),
            crossterm::style::Print("\x1B[0 q"),
            ResetColor,
            cursor::Show
        )?;

        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::MoveTo(0, 0)
        )?;

        // Final reset - guaranteed standard terminal
        execute!(
            self.stdout,
            crossterm::style::Print("\x1B[!p"),
            crossterm::style::Print("\x1B]12;white\x07"),
            crossterm::style::Print("\x1B[0 q"),
            crossterm::style::Print("\x1B[?25h"),
            ResetColor
        )?;

        self.stdout.flush()?;
        log::info!("{}", get_translation("terminal.cleanup.done", &[]));
        Ok(())
    }

    async fn disable_full_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode_enabled {
            return Ok(());
        }

        execute!(
            self.stdout,
            crossterm::style::Print("\x1B[?1000l"),
            crossterm::style::Print("\x1B[?1002l"),
            crossterm::style::Print("\x1B[?1015l"),
            crossterm::style::Print("\x1B[?1006l"),
            crossterm::style::Print("\x1B[?1049l")
        )?;

        disable_raw_mode()?;
        self.raw_mode_enabled = false;
        Ok(())
    }

    pub fn is_raw_mode_enabled(&self) -> bool {
        self.raw_mode_enabled
    }

    pub async fn force_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode_enabled {
            log::warn!("Raw mode was lost, re-enabling...");
            self.enable_full_raw_mode().await?;
        }
        Ok(())
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        if self.raw_mode_enabled {
            let _ = disable_raw_mode();
            let _ = execute!(
                std::io::stdout(),
                terminal::LeaveAlternateScreen,
                cursor::Show,
                ResetColor
            );
            log::warn!("Emergency terminal cleanup in destructor");
        }
    }
}
