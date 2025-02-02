// src/main.rs
use log::info;
use rush_sync::{error, run, test_logging};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Zuerst den Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        println!("Logger konnte nicht initialisiert werden: {}", e);
    }

    // Dann erst die Test-Nachrichten
    test_logging();

    info!("Anwendung gestartet");

    // Hauptanwendung starten
    run().await
}
