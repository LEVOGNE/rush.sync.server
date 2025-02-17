// src/main.rs
use rush_sync::{error, i18n, run};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        eprintln!("Logger-Initialisierung fehlgeschlagen: {}", e);
    }

    // Sprache initialisieren
    match i18n::init().await {
        Ok(_) => log::info!("Sprache erfolgreich initialisiert"),
        Err(e) => log::warn!("Sprachinitialisierung fehlgeschlagen: {}", e),
    }

    // Starte die Anwendung
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Fehler beim Ausführen: {}", e);
            Err(e)
        }
    }
}
