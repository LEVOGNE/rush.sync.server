// In einer neuen Datei: src/constants.rs
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG_PATHS: Vec<&'static str> = {
        #[cfg(debug_assertions)]
        {
            vec!["rush.toml", "src/rush.toml"]
        }
        #[cfg(not(debug_assertions))]
        {
            vec!["rush.toml", "./config/rush.toml", "../rush.toml"]
        }
    };
}

// Zentrale Terminal-Konfiguration
pub const APP_TITLE: &str = "RUSH SYNC";
pub const DEFAULT_BUFFER_SIZE: usize = 100;
pub const DEFAULT_POLL_RATE: u64 = 16;
pub const DOUBLE_ESC_THRESHOLD: u64 = 250;
