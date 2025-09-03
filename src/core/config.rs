// src/core/config.rs - Cleaned and simplified
use crate::core::constants::{DEFAULT_BUFFER_SIZE, DEFAULT_POLL_RATE};
use crate::core::prelude::*;
use crate::proxy::types::{ProxyConfig, ProxyConfigToml};
use crate::ui::color::AppColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// TOML Configuration Structure
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    general: GeneralConfig,
    #[serde(default)]
    server: Option<ServerConfigToml>,
    #[serde(default)]
    logging: Option<LoggingConfigToml>,
    #[serde(default)]
    theme: Option<HashMap<String, ThemeDefinitionConfig>>,
    language: LanguageConfig,
    proxy: Option<ProxyConfigToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    max_messages: usize,
    typewriter_delay: u64,
    input_max_length: usize,
    max_history: usize,
    poll_rate: u64,
    log_level: String,
    #[serde(default = "default_theme")]
    current_theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LanguageConfig {
    current: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ServerConfigToml {
    #[serde(default = "default_port_start")]
    port_range_start: u16,
    #[serde(default = "default_port_end")]
    port_range_end: u16,
    #[serde(default = "default_max_concurrent")]
    max_concurrent: usize,
    #[serde(default = "default_shutdown_timeout")]
    shutdown_timeout: u64,
    #[serde(default = "default_startup_delay")]
    startup_delay_ms: u64,
    #[serde(default = "default_workers")]
    workers: usize,
    #[serde(default = "default_auto_open_browser")]
    auto_open_browser: bool,

    // TLS Configuration
    #[serde(default = "default_enable_https")]
    enable_https: bool,
    #[serde(default = "default_https_port_offset")]
    https_port_offset: u16,
    #[serde(default = "default_cert_dir")]
    cert_dir: String,
    #[serde(default = "default_auto_cert")]
    auto_cert: bool,
    #[serde(default = "default_cert_validity_days")]
    cert_validity_days: u32,

    // Production Settings
    #[serde(default = "default_use_lets_encrypt")]
    use_lets_encrypt: bool,
    #[serde(default = "default_production_domain")]
    production_domain: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoggingConfigToml {
    #[serde(default = "default_max_file_size")]
    max_file_size_mb: u64,
    #[serde(default = "default_max_archive_files")]
    max_archive_files: u8,
    #[serde(default = "default_compress_archives")]
    compress_archives: bool,
    #[serde(default = "default_log_requests")]
    log_requests: bool,
    #[serde(default = "default_log_security")]
    log_security_alerts: bool,
    #[serde(default = "default_log_performance")]
    log_performance: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ThemeDefinitionConfig {
    input_text: String,
    input_bg: String,
    output_text: String,
    output_bg: String,
    #[serde(default = "default_prefix")]
    input_cursor_prefix: String,
    #[serde(default = "default_input_color")]
    input_cursor_color: String,
    #[serde(default = "default_cursor")]
    input_cursor: String,
    #[serde(default = "default_cursor")]
    output_cursor: String,
    #[serde(default = "default_output_color")]
    output_cursor_color: String,
}

// Default Functions
fn default_theme() -> String {
    "dark".into()
}
fn default_prefix() -> String {
    "/// ".into()
}
fn default_input_color() -> String {
    "LightBlue".into()
}
fn default_output_color() -> String {
    "White".into()
}
fn default_cursor() -> String {
    "PIPE".into()
}

// Server Defaults
fn default_port_start() -> u16 {
    8080
}
fn default_port_end() -> u16 {
    8180
}
fn default_max_concurrent() -> usize {
    10
}
fn default_shutdown_timeout() -> u64 {
    5
}
fn default_startup_delay() -> u64 {
    500
}
fn default_workers() -> usize {
    1
}
fn default_auto_open_browser() -> bool {
    true
}

// TLS Defaults
fn default_enable_https() -> bool {
    true
}
fn default_https_port_offset() -> u16 {
    1000
}
fn default_cert_dir() -> String {
    ".rss/certs".to_string()
}
fn default_auto_cert() -> bool {
    true
}
fn default_cert_validity_days() -> u32 {
    365
}
fn default_use_lets_encrypt() -> bool {
    false
}
fn default_production_domain() -> String {
    "localhost".to_string()
}

// Logging Defaults
fn default_max_file_size() -> u64 {
    100
}
fn default_max_archive_files() -> u8 {
    9
}
fn default_compress_archives() -> bool {
    true
}
fn default_log_requests() -> bool {
    true
}
fn default_log_security() -> bool {
    true
}
fn default_log_performance() -> bool {
    true
}

// Main Configuration Structures
#[derive(Clone)]
pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize,
    pub poll_rate: Duration,
    pub log_level: String,
    pub theme: Theme,
    pub current_theme_name: String,
    pub language: String,
    pub debug_info: Option<String>,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub proxy: ProxyConfig,
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub max_concurrent: usize,
    pub shutdown_timeout: u64,
    pub startup_delay_ms: u64,
    pub workers: usize,
    pub auto_open_browser: bool,

    // TLS Configuration
    pub enable_https: bool,
    pub https_port_offset: u16,
    pub cert_dir: String,
    pub auto_cert: bool,
    pub cert_validity_days: u32,
    pub use_lets_encrypt: bool,
    pub production_domain: String,
}

#[derive(Clone)]
pub struct LoggingConfig {
    pub max_file_size_mb: u64,
    pub max_archive_files: u8,
    pub compress_archives: bool,
    pub log_requests: bool,
    pub log_security_alerts: bool,
    pub log_performance: bool,
}

#[derive(Clone)]
pub struct Theme {
    pub input_text: AppColor,
    pub input_bg: AppColor,
    pub output_text: AppColor,
    pub output_bg: AppColor,
    pub input_cursor_prefix: String,
    pub input_cursor_color: AppColor,
    pub input_cursor: String,
    pub output_cursor: String,
    pub output_cursor_color: AppColor,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            input_text: AppColor::new(Color::White),
            input_bg: AppColor::new(Color::Black),
            output_text: AppColor::new(Color::White),
            output_bg: AppColor::new(Color::Black),
            input_cursor_prefix: "/// ".into(),
            input_cursor_color: AppColor::new(Color::LightBlue),
            input_cursor: "PIPE".into(),
            output_cursor: "PIPE".into(),
            output_cursor_color: AppColor::new(Color::White),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port_range_start: 8080,
            port_range_end: 8180,
            max_concurrent: 10,
            shutdown_timeout: 5,
            startup_delay_ms: 500,
            workers: 1,
            auto_open_browser: true,
            enable_https: true,
            https_port_offset: 1000,
            cert_dir: ".rss/certs".to_string(),
            auto_cert: true,
            cert_validity_days: 365,
            use_lets_encrypt: false,
            production_domain: "localhost".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 100,
            max_archive_files: 9,
            compress_archives: true,
            log_requests: true,
            log_security_alerts: true,
            log_performance: true,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        Self::load_with_messages(true).await
    }

    pub async fn load_with_messages(show_messages: bool) -> Result<Self> {
        // Try existing configs
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                if let Ok(config) = Self::from_file(&path).await {
                    if show_messages {
                        Self::log_startup(&config);
                    }
                    Self::apply_language(&config).await;
                    return Ok(config);
                }
            }
        }

        // Create new config
        let path = crate::setup::setup_toml::ensure_config_exists().await?;
        let mut config = Self::from_file(&path).await?;

        if show_messages {
            config.debug_info = Some(format!("New config: {}", path.display()));
            Self::log_startup(&config);
        }

        Self::apply_language(&config).await;
        Ok(config)
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(AppError::Io)?;
        let file: ConfigFile =
            toml::from_str(&content).map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

        let poll_rate = Self::clamp(file.general.poll_rate, 16, 1000, 16);
        let typewriter = Self::clamp(file.general.typewriter_delay, 0, 2000, 50);
        let theme = Self::load_theme(&file).unwrap_or_default();

        // Load server config
        let server = file
            .server
            .map_or_else(ServerConfig::default, |s| ServerConfig {
                port_range_start: s.port_range_start,
                port_range_end: s.port_range_end,
                max_concurrent: s.max_concurrent,
                shutdown_timeout: s.shutdown_timeout,
                startup_delay_ms: s.startup_delay_ms,
                workers: s.workers,
                auto_open_browser: s.auto_open_browser,
                enable_https: s.enable_https,
                https_port_offset: s.https_port_offset,
                cert_dir: s.cert_dir,
                auto_cert: s.auto_cert,
                cert_validity_days: s.cert_validity_days,
                use_lets_encrypt: s.use_lets_encrypt,
                production_domain: s.production_domain,
            });

        // Load logging config
        let logging = file
            .logging
            .map_or_else(LoggingConfig::default, |l| LoggingConfig {
                max_file_size_mb: l.max_file_size_mb,
                max_archive_files: l.max_archive_files,
                compress_archives: l.compress_archives,
                log_requests: l.log_requests,
                log_security_alerts: l.log_security_alerts,
                log_performance: l.log_performance,
            });

        let config = Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: file.general.max_messages,
            typewriter_delay: Duration::from_millis(typewriter),
            input_max_length: file.general.input_max_length,
            max_history: file.general.max_history,
            poll_rate: Duration::from_millis(poll_rate),
            log_level: file.general.log_level,
            theme,
            current_theme_name: file.general.current_theme,
            language: file.language.current,
            debug_info: None,
            server,
            logging,
            proxy: file.proxy.map(ProxyConfig::from).unwrap_or_default(),
        };

        // Auto-save corrected values
        if poll_rate != file.general.poll_rate || typewriter != file.general.typewriter_delay {
            let _ = config.save().await;
        }

        Ok(config)
    }

    pub async fn save(&self) -> Result<()> {
        let Some(path) = &self.config_path else {
            return Ok(());
        };

        let themes = Self::load_existing_themes().await.unwrap_or_default();
        let file = ConfigFile {
            general: GeneralConfig {
                max_messages: self.max_messages,
                typewriter_delay: self.typewriter_delay.as_millis() as u64,
                input_max_length: self.input_max_length,
                max_history: self.max_history,
                poll_rate: self.poll_rate.as_millis() as u64,
                log_level: self.log_level.clone(),
                current_theme: self.current_theme_name.clone(),
            },
            server: Some(ServerConfigToml {
                port_range_start: self.server.port_range_start,
                port_range_end: self.server.port_range_end,
                max_concurrent: self.server.max_concurrent,
                shutdown_timeout: self.server.shutdown_timeout,
                startup_delay_ms: self.server.startup_delay_ms,
                workers: self.server.workers,
                auto_open_browser: self.server.auto_open_browser,
                enable_https: self.server.enable_https,
                https_port_offset: self.server.https_port_offset,
                cert_dir: self.server.cert_dir.clone(),
                auto_cert: self.server.auto_cert,
                cert_validity_days: self.server.cert_validity_days,
                use_lets_encrypt: self.server.use_lets_encrypt,
                production_domain: self.server.production_domain.clone(),
            }),
            logging: Some(LoggingConfigToml {
                max_file_size_mb: self.logging.max_file_size_mb,
                max_archive_files: self.logging.max_archive_files,
                compress_archives: self.logging.compress_archives,
                log_requests: self.logging.log_requests,
                log_security_alerts: self.logging.log_security_alerts,
                log_performance: self.logging.log_performance,
            }),
            theme: if themes.is_empty() {
                None
            } else {
                Some(themes)
            },
            language: LanguageConfig {
                current: self.language.clone(),
            },
            proxy: Some(self.proxy.clone().into()),
        };

        let content = toml::to_string_pretty(&file)
            .map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

        // Ensure dir exists
        if let Some(parent) = std::path::PathBuf::from(path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        tokio::fs::write(path, content).await.map_err(AppError::Io)
    }

    pub async fn change_theme(&mut self, name: &str) -> Result<()> {
        let themes = Self::load_existing_themes().await?;
        let def = themes
            .get(name)
            .ok_or_else(|| AppError::Validation(format!("Theme '{}' not found", name)))?;

        self.theme = Theme::from_config(def)?;
        self.current_theme_name = name.into();
        self.save().await
    }

    pub fn get_performance_info(&self) -> String {
        let fps = 1000.0 / self.poll_rate.as_millis() as f64;
        let typewriter = if self.typewriter_delay.as_millis() > 0 {
            1000.0 / self.typewriter_delay.as_millis() as f64
        } else {
            f64::INFINITY
        };
        format!(
            "Performance: {:.1} FPS, Typewriter: {:.1} chars/sec, Max Servers: {}",
            fps, typewriter, self.server.max_concurrent
        )
    }

    // Helper methods
    fn clamp(value: u64, min: u64, max: u64, default: u64) -> u64 {
        if value < min || value > max {
            default
        } else {
            value
        }
    }

    fn load_theme(file: &ConfigFile) -> Option<Theme> {
        let themes = file.theme.as_ref()?;
        let def = themes.get(&file.general.current_theme)?;
        Theme::from_config(def).ok()
    }

    async fn load_existing_themes() -> Result<HashMap<String, ThemeDefinitionConfig>> {
        for path in crate::setup::setup_toml::get_config_paths() {
            if path.exists() {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AppError::Io)?;
                let file: ConfigFile = toml::from_str(&content)
                    .map_err(|e| AppError::Validation(format!("TOML: {}", e)))?;

                if let Some(themes) = file.theme {
                    return Ok(themes);
                }
            }
        }
        Ok(HashMap::new())
    }

    async fn apply_language(config: &Config) {
        let _ = crate::commands::lang::LanguageService::new()
            .load_and_apply_from_config(config)
            .await;
    }

    fn log_startup(config: &Config) {
        if config.poll_rate.as_millis() < 16 {
            log::warn!("Performance: poll_rate sehr niedrig!");
        }
        log::info!("Rush Sync Server v{}", crate::core::constants::VERSION);
        log::info!(
            "Server Config: Ports {}-{}, Max: {}",
            config.server.port_range_start,
            config.server.port_range_end,
            config.server.max_concurrent
        );
    }
}

impl Theme {
    fn from_config(def: &ThemeDefinitionConfig) -> Result<Self> {
        Ok(Self {
            input_text: AppColor::from_string(&def.input_text)?,
            input_bg: AppColor::from_string(&def.input_bg)?,
            output_text: AppColor::from_string(&def.output_text)?,
            output_bg: AppColor::from_string(&def.output_bg)?,
            input_cursor_prefix: def.input_cursor_prefix.clone(),
            input_cursor_color: AppColor::from_string(&def.input_cursor_color)?,
            input_cursor: def.input_cursor.clone(),
            output_cursor: def.output_cursor.clone(),
            output_cursor_color: AppColor::from_string(&def.output_cursor_color)?,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            max_messages: DEFAULT_BUFFER_SIZE,
            typewriter_delay: Duration::from_millis(50),
            input_max_length: DEFAULT_BUFFER_SIZE,
            max_history: 30,
            poll_rate: Duration::from_millis(DEFAULT_POLL_RATE),
            log_level: "info".into(),
            theme: Theme::default(),
            current_theme_name: "dark".into(),
            language: crate::i18n::DEFAULT_LANGUAGE.into(),
            debug_info: None,
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            proxy: ProxyConfig::default(),
        }
    }
}
