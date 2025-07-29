// =====================================================
// FILE: src/setup/setup_toml.rs - KORRIGIERTE TOML STRUKTUR
// =====================================================

use crate::core::prelude::*;
use std::path::PathBuf;
use tokio::fs;

// ✅ KORRIGIERTE DEFAULT-CONFIG mit neuer Struktur
const DEFAULT_CONFIG: &str = r#"[general]
max_messages = 100
# Typewriter-Effekt: 50ms = 20 Zeichen/Sekunde (empfohlen: 30-100ms)
typewriter_delay = 5
input_max_length = 100
max_history = 30
# Poll-Rate: 16ms = 60 FPS (empfohlen: 16-33ms, NICHT unter 16!)
poll_rate = 16
log_level = "info"
current_theme = "dark"

[language]
current = "en"

# =================================================================
# THEME-DEFINITIONEN (mit integrierten Prompts)
# =================================================================

[theme.dark]
input_text = "Black"
input_bg = "White"
cursor = "Black"
output_text = "White"
output_bg = "Black"
prompt_text = "/// "
prompt_color = "Black"

[theme.light]
input_text = "White"
input_bg = "Black"
cursor = "White"
output_text = "Black"
output_bg = "White"
prompt_text = "/// "
prompt_color = "White"

[theme.green]
input_text = "LightGreen"
input_bg = "Black"
cursor = "LightGreen"
output_text = "Green"
output_bg = "Black"
prompt_text = "$ "
prompt_color = "LightGreen"

[theme.blue]
input_text = "White"
input_bg = "Blue"
cursor = "White"
output_text = "LightBlue"
output_bg = "White"
prompt_text = "> "
prompt_color = "White"

# =================================================================
# HINWEIS: PROMPT IST JETZT TEIL DER THEMES!
# =================================================================
#
# Jedes Theme definiert:
#   - prompt_text  (Der Prompt-String z.B. "/// " oder "λ> ")
#   - prompt_color (Die Prompt-Farbe z.B. "LightBlue")
#
# Dies löst das macOS Terminal Schwarz-Problem und macht
# Prompts thematisch konsistent.
#
# ENTFERNT: [prompt] Section (war vorher separiert)
#
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
#
# current_theme:
#   - "dark" = Dunkles Theme (Standard)
#   - "light" = Helles Theme
#   - "matrix" = Matrix-Style (grün)
#   - "blue" = Blaues Theme
#   - "hacker" = Hacker-Style (grün/rot)
#   - "minimal" = Minimalistisches Theme
#
# NEUE FEATURE: THEME-INTEGRIERTE PROMPTS
#   - Jedes Theme hat eigenen prompt_text und prompt_color
#   - Löst macOS Terminal Schwarz-Problem
#   - Thematisch konsistente Prompts
# ================================================================="#;

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
        log::info!(
            "✨ NEU: Prompt ist jetzt Teil der Themes! Jedes Theme hat eigenen Prompt-Style."
        );
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
