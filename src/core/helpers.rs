use crate::core::prelude::*;

/// Runtime-sicherer Config-Loader
pub fn get_config() -> Result<Config> {
    // Einfachste Lösung: Channel-basiert
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(Config::load());
        let _ = tx.send(result);
    });

    rx.recv()
        .map_err(|_| AppError::Validation("Config loading timeout".to_string()))?
}
