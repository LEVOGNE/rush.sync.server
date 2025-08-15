// =====================================================
// FILE: src/ui/terminal.rs - FIXED OHNE FUTURES DEPENDENCY
// =====================================================

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
        let stdout = io::stdout();
        Ok(Self {
            stdout,
            raw_mode_enabled: false,
        })
    }

    pub async fn setup(&mut self) -> Result<()> {
        // ✅ SCHRITT 1: VOLLSTÄNDIGER RAW MODE
        self.enable_full_raw_mode().await?;

        // ✅ SCHRITT 2: TERMINAL SETUP
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            EnterAlternateScreen,
            terminal::DisableLineWrap,
            terminal::SetTitle(APP_TITLE),
            // Cursor-Zustand beim Setup zurücksetzen
            crossterm::style::Print("\x1B]112\x07"), // Reset cursor color
            crossterm::style::Print("\x1B[0 q"),     // Reset cursor shape
            cursor::Hide
        )?;

        Ok(())
    }

    /// ✅ VOLLSTÄNDIGER RAW MODE - Übernimmt ALLE Tastatur-Events
    async fn enable_full_raw_mode(&mut self) -> Result<()> {
        // Standard Raw Mode aktivieren
        enable_raw_mode()?;
        self.raw_mode_enabled = true;

        // ✅ ERWEITERTE TERMINAL-KONTROLLE
        execute!(
            self.stdout,
            // Alle Terminal-Escape-Sequenzen deaktivieren
            crossterm::style::Print("\x1B[?1000h"), // Mouse tracking an (optional)
            crossterm::style::Print("\x1B[?1002h"), // Cell motion mouse tracking
            crossterm::style::Print("\x1B[?1015h"), // Enable urxvt mouse mode
            crossterm::style::Print("\x1B[?1006h"), // Enable SGR mouse mode
            // Spezielle Key-Kombinationen abfangen
            crossterm::style::Print("\x1B[?1049h"), // Enable alternative screen buffer
        )?;
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        log::info!("🔄 Starting terminal cleanup...");

        // ✅ SCHRITT 1: Raw Mode zuerst deaktivieren
        if self.raw_mode_enabled {
            self.disable_full_raw_mode().await?;
        }

        // ✅ SCHRITT 2: Cursor-Farbe KOMPLETT zurücksetzen
        execute!(
            self.stdout,
            // Reset cursor color (multiple standards for maximum compatibility)
            crossterm::style::Print("\x1B]12;\x07"), // Xterm: empty = default
            crossterm::style::Print("\x1B]Pl\x1B\\"), // iTerm2: reset
            crossterm::style::Print("\x1B]112\x07"), // OSC 112: reset cursor color
            crossterm::style::Print("\x1B[0 q"),     // Reset cursor shape to default
            ResetColor,                              // Reset ANSI colors
            cursor::Show,                            // Show cursor
        )?;

        // ✅ SCHRITT 3: Terminal-Modi zurücksetzen
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::MoveTo(0, 0)
        )?;

        // ✅ SCHRITT 4: FINAL RESET - garantiert Standard-Terminal
        execute!(
            self.stdout,
            // Kompletter Reset aller Terminal-Modi
            crossterm::style::Print("\x1B[!p"), // RIS - Reset to Initial State
            crossterm::style::Print("\x1B]12;white\x07"), // Explicit white cursor color
            crossterm::style::Print("\x1B[0 q"), // Default cursor shape
            crossterm::style::Print("\x1B[?25h"), // Show cursor
            ResetColor,                         // Final color reset
        )?;

        // Buffer explizit leeren
        self.stdout.flush()?;

        log::info!("{}", get_translation("terminal.cleanup.done", &[]));
        Ok(())
    }

    /// ✅ ERWEITERTEN RAW MODE DEAKTIVIEREN
    async fn disable_full_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode_enabled {
            return Ok(());
        }

        // Erweiterte Modi deaktivieren
        execute!(
            self.stdout,
            // Mouse tracking deaktivieren
            crossterm::style::Print("\x1B[?1000l"), // Mouse tracking off
            crossterm::style::Print("\x1B[?1002l"), // Cell motion mouse tracking off
            crossterm::style::Print("\x1B[?1015l"), // Disable urxvt mouse mode
            crossterm::style::Print("\x1B[?1006l"), // Disable SGR mouse mode
            // Alternative screen buffer deaktivieren
            crossterm::style::Print("\x1B[?1049l"), // Disable alternative screen buffer
        )?;

        // Standard Raw Mode deaktivieren
        disable_raw_mode()?;
        self.raw_mode_enabled = false;
        Ok(())
    }

    /// ✅ DEBUG: Prüfe ob Raw Mode aktiv ist
    pub fn is_raw_mode_enabled(&self) -> bool {
        self.raw_mode_enabled
    }

    /// ✅ FORCE RAW MODE (falls es während der Laufzeit verloren geht)
    pub async fn force_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode_enabled {
            log::warn!("🚨 Raw mode was lost, re-enabling...");
            self.enable_full_raw_mode().await?;
        }
        Ok(())
    }
}

// ✅ SICHERER DESTRUCTOR OHNE FUTURES
impl Drop for TerminalManager {
    fn drop(&mut self) {
        if self.raw_mode_enabled {
            // ✅ SYNC CLEANUP (ohne futures::executor)
            let _ = disable_raw_mode();
            let _ = execute!(
                std::io::stdout(),
                terminal::LeaveAlternateScreen,
                cursor::Show,
                ResetColor
            );
            log::warn!("🚨 Emergency terminal cleanup in destructor");
        }
    }
}
