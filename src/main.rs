// src/main.rs
use log::info;
use rush_sync::core::constants::VERSION;
use rush_sync::i18n;
use rush_sync::{error, run};

// In main.rs
#[tokio::main]
async fn main() -> error::Result<()> {
    if let Err(e) = rush_sync::output::logging::init() {
        println!("Logger konnte nicht initialisiert werden: {}", e);
    }

    // Stille Initialisierung der Sprache
    if let Err(e) = i18n::init_language_silent().await {
        log::warn!("Fehler beim Laden der Spracheinstellung: {}", e);
    }

    let version_msg = i18n::get_translation("system.startup.version", &[VERSION]);
    info!("{}", version_msg);

    run().await
}
