// =====================================================
// FILE: src/commands/performance/manager.rs - BUSINESS LOGIC
// =====================================================

use crate::core::prelude::*;

/// Zentrale Verwaltung aller Performance-Operationen
pub struct PerformanceManager;

impl PerformanceManager {
    /// Zeigt vollständigen Performance-Status
    pub fn get_status() -> Result<String> {
        let config_data = Self::load_config_values()?;
        let performance_analysis = Self::analyze_performance(&config_data);

        // ✅ TEST 1: Einfache Version OHNE i18n
        let simple_report = format!(
            "SIMPLE PERFORMANCE:\nPoll: {}ms\nTypewriter: {}ms\nMessages: {}\nHistory: {}",
            config_data.poll_rate,
            config_data.typewriter_delay,
            config_data.max_messages,
            config_data.max_history
        );

        // ✅ TEST 2: Deutsche i18n Version (problematisch?)
        let i18n_report = Self::format_status_report(&config_data, &performance_analysis);

        // ✅ DEBUG: Zähle Zeilen in beiden Versionen
        let simple_lines = simple_report.lines().count();
        let i18n_lines = i18n_report.lines().count();

        log::info!("📊 PERFORMANCE DEBUG:");
        log::info!("   Simple version: {} lines", simple_lines);
        log::info!("   i18n version: {} lines", i18n_lines);
        log::info!(
            "   First 100 chars of i18n: {}",
            &i18n_report.chars().take(100).collect::<String>()
        );

        // ✅ TEST: Verwende einfache Version zum Testen
        if simple_lines <= 6 {
            Ok(simple_report)
        } else {
            // Falls das auch Probleme macht, ultraminimal:
            Ok(format!(
                "Performance: {}ms poll, {}ms typewriter",
                config_data.poll_rate, config_data.typewriter_delay
            ))
        }
    }
    /// Lädt Config-Werte direkt aus Datei (robust & schnell)
    fn load_config_values() -> Result<ConfigData> {
        let paths = crate::setup::setup_toml::get_config_paths();

        for path in paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // ✅ CLIPPY FIX: Initialisierung mit struct literal statt Default + assignment
                    let mut config_data = ConfigData {
                        config_path: path.display().to_string(),
                        config_name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        ..Default::default() // ✅ Rest mit Default füllen
                    };

                    // ✅ ROBUSTES PARSING (Line-by-Line) bleibt gleich
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("poll_rate") && line.contains('=') {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(value) = value_str.trim().parse::<u64>() {
                                    config_data.poll_rate = value;
                                }
                            }
                        } else if line.starts_with("typewriter_delay") && line.contains('=') {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(value) = value_str.trim().parse::<u64>() {
                                    config_data.typewriter_delay = value;
                                }
                            }
                        } else if line.starts_with("max_messages") && line.contains('=') {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(value) = value_str.trim().parse::<u64>() {
                                    config_data.max_messages = value;
                                }
                            }
                        } else if line.starts_with("max_history") && line.contains('=') {
                            if let Some(value_str) = line.split('=').nth(1) {
                                if let Ok(value) = value_str.trim().parse::<u64>() {
                                    config_data.max_history = value;
                                }
                            }
                        } else if line.starts_with("log_level") && line.contains('=') {
                            if let Some(value_str) = line.split('=').nth(1) {
                                config_data.log_level =
                                    value_str.trim().trim_matches('"').trim().to_string();
                            }
                        }
                    }

                    return Ok(config_data);
                }
            }
        }

        Err(AppError::Validation(
            "Keine Config-Datei gefunden".to_string(),
        ))
    }

    /// Analysiert Performance-Werte und gibt Bewertung zurück
    fn analyze_performance(config: &ConfigData) -> PerformanceAnalysis {
        let fps = 1000.0 / config.poll_rate as f64;

        let poll_status = match config.poll_rate {
            0 => PerformanceStatus::Critical,
            1..=15 => PerformanceStatus::Critical,
            16..=33 => PerformanceStatus::Optimal,
            34..=50 => PerformanceStatus::Good,
            51..=100 => PerformanceStatus::Slow,
            _ => PerformanceStatus::VerySlow,
        };

        let typewriter_info = if config.typewriter_delay == 0 {
            TypewriterInfo {
                chars_per_sec: None,
                is_active: false,
            }
        } else {
            let chars_per_sec = 1000.0 / config.typewriter_delay as f64;
            TypewriterInfo {
                chars_per_sec: Some(chars_per_sec),
                is_active: true,
            }
        };

        PerformanceAnalysis {
            fps,
            poll_status,
            typewriter_info,
        }
    }

    /// Formatiert den vollständigen Status-Report (i18n-fähig)
    fn format_status_report(config: &ConfigData, analysis: &PerformanceAnalysis) -> String {
        let status_icon = match analysis.poll_status {
            PerformanceStatus::Critical => "🔥",
            PerformanceStatus::Optimal => "✅",
            PerformanceStatus::Good => "✅",
            PerformanceStatus::Slow => "⚠️",
            PerformanceStatus::VerySlow => "❌",
        };

        let typewriter_icon = if analysis.typewriter_info.is_active {
            "⌨️"
        } else {
            "🚫"
        };

        // ✅ i18n STRINGS verwenden statt hardcoded
        let header = crate::i18n::get_translation("system.commands.performance.header", &[]);
        let poll_label = crate::i18n::get_translation("system.commands.performance.poll_rate", &[]);
        let typewriter_label =
            crate::i18n::get_translation("system.commands.performance.typewriter", &[]);
        let messages_label =
            crate::i18n::get_translation("system.commands.performance.max_messages", &[]);
        let history_label =
            crate::i18n::get_translation("system.commands.performance.max_history", &[]);
        let log_label = crate::i18n::get_translation("system.commands.performance.log_level", &[]);
        let config_label = crate::i18n::get_translation("system.commands.performance.config", &[]);
        let recommendations =
            crate::i18n::get_translation("system.commands.performance.recommendations", &[]);
        let commands_label =
            crate::i18n::get_translation("system.commands.performance.related_commands", &[]);
        let config_edit =
            crate::i18n::get_translation("system.commands.performance.config_edit", &[]);
        let log_change =
            crate::i18n::get_translation("system.commands.performance.log_change", &[]);
        let version_cmd =
            crate::i18n::get_translation("system.commands.performance.version_cmd", &[]);
        let restart_cmd =
            crate::i18n::get_translation("system.commands.performance.restart_cmd", &[]);

        // ✅ STATUS MESSAGE übersetzt
        let status_message = match analysis.poll_status {
            PerformanceStatus::Critical => {
                crate::i18n::get_translation("system.commands.performance.status.critical", &[])
            }
            PerformanceStatus::Optimal => {
                crate::i18n::get_translation("system.commands.performance.status.optimal", &[])
            }
            PerformanceStatus::Good => {
                crate::i18n::get_translation("system.commands.performance.status.good", &[])
            }
            PerformanceStatus::Slow => {
                crate::i18n::get_translation("system.commands.performance.status.slow", &[])
            }
            PerformanceStatus::VerySlow => {
                crate::i18n::get_translation("system.commands.performance.status.very_slow", &[])
            }
        };

        // ✅ TYPEWRITER STATUS übersetzt
        let typewriter_status = if analysis.typewriter_info.is_active {
            crate::i18n::get_translation(
                "system.commands.performance.typewriter.active",
                &[&format!(
                    "{:.1}",
                    analysis.typewriter_info.chars_per_sec.unwrap_or(0.0)
                )],
            )
        } else {
            crate::i18n::get_translation("system.commands.performance.typewriter.disabled", &[])
        };

        format!(
            "{}\n\n\
            🎯 {}: {}ms ({:.1} FPS) {} {}\n\
            {} {}: {}ms ({})\n\
            📈 {}: {}\n\
            📜 {}: {}\n\
            🎨 {}: {}\n\
            📍 {}: {}\n\n\
            💡 {}:\n\
            {}\n\n\
            🔧 {}:\n\
            • {}: {}\n\
            • {}\n\
            • {}: 'version'\n\
            • {}: 'restart'",
            header,
            poll_label,
            config.poll_rate,
            analysis.fps,
            status_icon,
            status_message,
            typewriter_icon,
            typewriter_label,
            config.typewriter_delay,
            typewriter_status,
            messages_label,
            config.max_messages,
            history_label,
            config.max_history,
            log_label,
            config.log_level.to_uppercase(),
            config_label,
            config.config_name,
            recommendations,
            crate::i18n::get_translation(
                "system.commands.performance.recommendations.content",
                &[]
            ),
            commands_label,
            config_edit,
            config.config_path,
            log_change,
            version_cmd,
            restart_cmd
        )
    }
}

// =====================================================
// DATENSTRUKTUREN
// =====================================================

#[derive(Debug, Default)]
struct ConfigData {
    poll_rate: u64,
    typewriter_delay: u64,
    max_messages: u64,
    max_history: u64,
    log_level: String,
    config_path: String,
    config_name: String,
}

#[derive(Debug)]
struct PerformanceAnalysis {
    fps: f64,
    poll_status: PerformanceStatus,
    typewriter_info: TypewriterInfo,
}

#[derive(Debug)]
enum PerformanceStatus {
    Critical,
    Optimal,
    Good,
    Slow,
    VerySlow,
}

#[derive(Debug)]
struct TypewriterInfo {
    chars_per_sec: Option<f64>,
    is_active: bool,
}
