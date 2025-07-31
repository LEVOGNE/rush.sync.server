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
}

impl TerminalManager {
    pub async fn new() -> Result<Self> {
        let stdout = io::stdout();
        Ok(Self { stdout })
    }

    pub async fn setup(&mut self) -> Result<()> {
        enable_raw_mode()?;

        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            EnterAlternateScreen,
            terminal::DisableLineWrap,
            terminal::SetTitle(APP_TITLE),
            cursor::Hide
        )?;
        log::info!("{}", get_translation("terminal.setup.done", &[]));

        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        // Erst Terminal-Modi zurücksetzen
        disable_raw_mode()?;

        // Dann alle cleanup operations in der richtigen Reihenfolge
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All), // NEU: Clear vor Leave
            LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::Show,
            ResetColor,
            cursor::MoveTo(0, 0)
        )?;

        // Buffer explizit leeren
        self.stdout.flush()?;

        log::info!("{}", get_translation("terminal.cleanup.done", &[]));

        Ok(())
    }
}
