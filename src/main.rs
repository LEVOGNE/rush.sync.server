// src/main.rs
pub mod color;
pub mod config;
pub mod constants;
pub mod cursor;
pub mod error;
pub mod event;
pub mod input;
pub mod keyboard;
pub mod logging;
pub mod message;
pub mod output;
pub mod prelude;
pub mod screen;
pub mod scroll;
pub mod terminal;
pub mod widget;

use config::Config;
use error::Result;
use log::info;
use screen::ScreenManager;

use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Logger initialisieren
    if let Err(e) = logging::init() {
        println!("Logger konnte nicht initialisiert werden: {}", e);
    }

    // Test-Logging
    log::error!("Das ist eine ERROR Test-Nachricht!");
    log::warn!("Das ist eine WARN Test-Nachricht!");
    log::info!("Das ist eine INFO Test-Nachricht!");
    log::debug!("Das ist eine DEBUG Test-Nachricht!");

    let config = Config::load().await?;

    info!("Anwendung gestartet");
    let mut screen = ScreenManager::new(&config).await?;
    screen.run().await
}
