// =====================================================
// FILE: src/setup/setup_toml.rs - SICHERE DEFAULTS
// =====================================================

use crate::core::prelude::*;
use std::path::PathBuf;
use tokio::fs;

// ✅ SICHERE DEFAULT-CONFIG mit Performance-Kommentaren
const DEFAULT_CONFIG: &str = r#"[general]
max_messages = 100
# Typewriter-Effekt: 50ms = 20 Zeichen/Sekunde (empfohlen: 30-100ms)
typewriter_delay = 50
input_max_length = 100
max_history = 30
# Poll-Rate: 16ms = 60 FPS (empfohlen: 16-33ms, NICHT unter 16!)
poll_rate = 16
log_level = "info"

[theme]
input_text = "Black"
input_bg = "White"
cursor = "Black"
output_text = "DarkGray"
output_bg = "Black"

[prompt]
text = "/// "
color = "Black"

[language]
current = "en"

# =================================================================
# PERFORMANCE-HINWEISE:
# =================================================================
# poll_rate:
#   - 16ms = 60 FPS (EMPFOHLEN für flüssiges UI)
#   - 33ms = 30 FPS (akzeptabel für langsamere Systeme)
#   - 1-15ms = NICHT empfohlen (hohe CPU-Last!)
#   - 0ms = CRASH! (Tokio interval panic)
#
# typewriter_delay:
#   - 50ms = 20 Zeichen/Sekunde (gut lesbar)
#   - 30ms = 33 Zeichen/Sekunde (schnell)
#   - 100ms = 10 Zeichen/Sekunde (langsam)
#   - 0ms = Typewriter-Effekt deaktiviert
# =================================================================
"#;

pub async fn ensure_config_exists() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().map_err(AppError::Io)?;
    let base_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::Validation(get_translation("system.config.dir_error", &[])))?;

    let config_dir = base_dir.join(".rss");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .await
            .map_err(AppError::Io)?;
        log::debug!(
            "{}",
            get_translation(
                "system.config.dir_created",
                &[&config_dir.display().to_string()]
            )
        );
    }

    let config_path = config_dir.join("rush.toml");
    if !config_path.exists() {
        fs::write(&config_path, DEFAULT_CONFIG)
            .await
            .map_err(AppError::Io)?;

        // ✅ INFO NUR BEI FIRST-RUN
        log::info!(
            "{}",
            get_translation(
                "system.config.file_created",
                &[&config_path.display().to_string()]
            )
        );

        // ✅ PERFORMANCE-TIPP für neue Nutzer
        log::info!("💡 Tipp: Für optimale Performance, behalte poll_rate >= 16ms");
    }

    Ok(config_path)
}

pub fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(base_dir) = exe_path.parent() {
            paths.push(base_dir.join(".rss/rush.toml"));
            paths.push(base_dir.join("rush.toml"));
            paths.push(base_dir.join("config/rush.toml"));
        }
    }
    #[cfg(debug_assertions)]
    {
        paths.push(PathBuf::from("rush.toml"));
        paths.push(PathBuf::from("src/rush.toml"));
    }
    paths
}
