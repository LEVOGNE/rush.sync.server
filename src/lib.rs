// src/lib.rs
pub mod commands;
pub mod core;
pub mod i18n;
pub mod input;
pub mod output;
pub mod setup;
pub mod ui;

// Re-exports für einfacheren Zugriff
pub use core::error;
pub use core::*;
pub use input::*;
pub use output::logging;
pub use output::*;
pub use setup::*;
pub use ui::*;

/// Führt Test-Logging aus
/* pub fn test_logging() {
    log::error!("Das ist eine ERROR Test-Nachricht!");
    log::warn!("Das ist eine WARN Test-Nachricht!");
    log::info!("Das ist eine INFO Test-Nachricht!");
    log::debug!("Das ist eine DEBUG Test-Nachricht!");
} */

/// Initialisiert die Anwendung und startet den Haupt-Loop
pub async fn run() -> error::Result<()> {
    // Konfiguration laden
    let config = match core::config::Config::load().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Fehler beim Laden der Konfiguration: {}", e);
            return Err(e);
        }
    };

    // Screen-Manager initialisieren
    let mut screen = match ui::screen::ScreenManager::new(&config).await {
        Ok(screen) => screen,
        Err(e) => {
            eprintln!("Fehler beim Initialisieren des Screen-Managers: {}", e);
            return Err(e);
        }
    };

    // Screen-Manager ausführen
    screen.run().await
}
