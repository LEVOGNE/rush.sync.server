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
    // Logger initialisieren
    if let Err(e) = logging::init() {
        println!("Logger konnte nicht initialisiert werden: {}", e);
    }

    // Konfiguration laden
    let config = core::config::Config::load().await?;

    // Screen-Manager initialisieren und starten
    let mut screen = ui::screen::ScreenManager::new(&config).await?;
    screen.run().await
}
