// src/main.rs
use log::info;
use rush_sync::core::constants::VERSION;
use rush_sync::{error, run};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Zuerst den Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        println!("Logger konnte nicht initialisiert werden: {}", e);
    }

    info!("Rush Sync Version {}", VERSION);

    // Hauptanwendung starten
    run().await
}
