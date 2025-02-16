// src/main.rs
//use crate::i18n::get_translation;
//use rush_sync::core::constants::VERSION;
use rush_sync::i18n;
//use rush_sync::ui::color::AppColor;
use rush_sync::{error, run};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        eprintln!("Logger-Initialisierung fehlgeschlagen: {}", e);
    }

    // Sprache initialisieren mit Fallback
    match i18n::init_language_silent().await {
        Ok(_) => log::debug!("Sprache erfolgreich initialisiert"),
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
