use crate::core::prelude::*;

pub struct PerformanceManager;

impl PerformanceManager {
    pub fn get_status() -> Result<String> {
        let config_data = Self::load_config_values()?;
        let performance_analysis = Self::analyze_performance(&config_data);

        Self::format_comprehensive_report(&config_data, &performance_analysis)
    }

    fn load_config_values() -> Result<ConfigData> {
        let paths = crate::setup::setup_toml::get_config_paths();

        for path in paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let mut config_data = ConfigData::default();

                    for line in content.lines() {
                        let line = line.trim();
                        Self::parse_config_line(line, &mut config_data);
                    }

                    return Ok(config_data);
                }
            }
        }

        Err(AppError::Validation("No config found".to_string()))
    }

    fn parse_config_line(line: &str, config_data: &mut ConfigData) {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "poll_rate" => {
                    if let Ok(val) = value.parse::<u64>() {
                        config_data.poll_rate = val;
                    }
                }
                "typewriter_delay" => {
                    if let Ok(val) = value.parse::<u64>() {
                        config_data.typewriter_delay = val;
                    }
                }
                "max_messages" => {
                    if let Ok(val) = value.parse::<u64>() {
                        config_data.max_messages = val;
                    }
                }
                "max_history" => {
                    if let Ok(val) = value.parse::<u64>() {
                        config_data.max_history = val;
                    }
                }
                "log_level" => {
                    config_data.log_level = value.to_string();
                }
                "current_theme" => {
                    config_data.current_theme = value.to_string();
                }
                _ => {}
            }
        }
    }

    fn analyze_performance(config: &ConfigData) -> PerformanceAnalysis {
        let fps = if config.poll_rate > 0 {
            1000.0 / config.poll_rate as f64
        } else {
            0.0
        };

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
            let chars_per_sec = if config.typewriter_delay > 0 {
                1000.0 / config.typewriter_delay as f64
            } else {
                f64::INFINITY // Unendlich schnell wenn delay = 0
            };
            TypewriterInfo {
                chars_per_sec: Some(chars_per_sec),
                is_active: true,
            }
        };

        let memory_usage = Self::estimate_memory_usage(config);

        PerformanceAnalysis {
            fps,
            poll_status,
            typewriter_info,
            memory_usage,
            recommendations: Self::generate_recommendations(config),
        }
    }

    fn estimate_memory_usage(config: &ConfigData) -> MemoryUsage {
        let message_buffer_mb = (config.max_messages * 100) as f64 / 1024.0 / 1024.0;
        let history_buffer_mb = (config.max_history * 50) as f64 / 1024.0 / 1024.0;
        let i18n_cache_mb = 0.5;

        MemoryUsage {
            total_estimated_mb: message_buffer_mb + history_buffer_mb + i18n_cache_mb,
            message_buffer_mb,
            history_buffer_mb,
            i18n_cache_mb,
        }
    }

    fn generate_recommendations(config: &ConfigData) -> Vec<String> {
        let mut recommendations = Vec::new();

        if config.poll_rate < 16 {
            recommendations
                .push("⚡ poll_rate < 16ms: Very high CPU load - recommended: 16-33ms".to_string());
        }

        if config.max_messages > 1000 {
            recommendations
                .push("💾 Too many messages in buffer - recommended: max 500".to_string());
        }

        if config.typewriter_delay > 0 && config.typewriter_delay < 10 {
            recommendations
                .push("⌨️ typewriter_delay < 10ms: Very fast - recommended: 30-100ms".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("✅ All settings optimally configured".to_string());
        }

        recommendations
    }

    fn format_comprehensive_report(
        config: &ConfigData,
        analysis: &PerformanceAnalysis,
    ) -> Result<String> {
        let mut report = String::new();

        report.push_str("📊 COMPREHENSIVE PERFORMANCE REPORT\n");
        report.push_str("=".repeat(50).as_str());
        report.push_str("\n\n");

        report.push_str("🎯 System Performance\n");
        report.push_str(&format!(
            "   • Poll Rate: {}ms ({:.1} FPS) {}\n",
            config.poll_rate,
            analysis.fps,
            Self::get_status_icon(&analysis.poll_status)
        ));

        if analysis.typewriter_info.is_active {
            report.push_str(&format!(
                "   • Typewriter Speed: {}ms ({:.1} chars/sec)\n",
                config.typewriter_delay,
                analysis.typewriter_info.chars_per_sec.unwrap_or(0.0)
            ));
        } else {
            report.push_str("   • Typewriter Speed: DISABLED\n");
        }

        report.push_str("\n💾 Memory Usage\n");
        report.push_str(&format!(
            "   • Total Estimated: {:.2} MB\n",
            analysis.memory_usage.total_estimated_mb
        ));
        report.push_str(&format!(
            "   • Message Buffer: {:.2} MB\n",
            analysis.memory_usage.message_buffer_mb
        ));
        report.push_str(&format!(
            "   • History Buffer: {:.2} MB\n",
            analysis.memory_usage.history_buffer_mb
        ));
        report.push_str(&format!(
            "   • i18n Cache: {:.2} MB\n",
            analysis.memory_usage.i18n_cache_mb
        ));

        report.push_str("\n💡 Recommendations\n");
        for recommendation in &analysis.recommendations {
            report.push_str(&format!("   • {}\n", recommendation));
        }

        report.push_str("\n🔧 Related Commands\n");
        report.push_str("   • log-level debug - Enable debug logging\n");

        Ok(report)
    }

    fn get_status_icon(status: &PerformanceStatus) -> &'static str {
        match status {
            PerformanceStatus::Critical => "🔥",
            PerformanceStatus::Optimal => "✅",
            PerformanceStatus::Good => "✅",
            PerformanceStatus::Slow => "⚠️",
            PerformanceStatus::VerySlow => "❌",
        }
    }
}

#[derive(Debug, Default)]
struct ConfigData {
    poll_rate: u64,
    typewriter_delay: u64,
    max_messages: u64,
    max_history: u64,
    log_level: String,
    current_theme: String,
}

#[derive(Debug)]
struct PerformanceAnalysis {
    fps: f64,
    poll_status: PerformanceStatus,
    typewriter_info: TypewriterInfo,
    memory_usage: MemoryUsage,
    recommendations: Vec<String>,
}

#[derive(Debug)]
struct MemoryUsage {
    total_estimated_mb: f64,
    message_buffer_mb: f64,
    history_buffer_mb: f64,
    i18n_cache_mb: f64,
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
