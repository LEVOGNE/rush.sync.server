use crate::constants::APP_TITLE;
use crate::prelude::*;
use crossterm::{
    cursor, execute,
    style::ResetColor,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

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

        /* if crossterm::tty::IsTty::is_tty(&self.stdout) {
            log::debug!("Terminal unterstützt ANSI-Farben");
        } else {
            log::warn!("Terminal unterstützt möglicherweise keine ANSI-Farben!");
        } */

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

        Ok(())
    }
}
