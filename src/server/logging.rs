// Complete Updated src/server/logging.rs
use crate::core::config::LoggingConfig;
use crate::core::prelude::*;
use actix_web::HttpMessage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerLogEntry {
    pub timestamp: String,
    pub timestamp_unix: u64,
    pub event_type: LogEventType,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub bytes_sent: Option<u64>,
    pub referer: Option<String>,
    pub query_string: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LogEventType {
    Request,
    ServerStart,
    ServerStop,
    ServerError,
    SecurityAlert,
    PerformanceWarning,
}

#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    pub max_file_size_bytes: u64,
    pub max_archive_files: u8,
    pub compress_archives: bool,
}

impl LogRotationConfig {
    // NEW: Create from main LoggingConfig
    pub fn from_main_config(logging_config: &LoggingConfig) -> Self {
        Self {
            max_file_size_bytes: logging_config.max_file_size_mb * 1024 * 1024, // Convert MB to bytes
            max_archive_files: logging_config.max_archive_files,
            compress_archives: logging_config.compress_archives,
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 100 * 1024 * 1024, // 100MB
            max_archive_files: 9,
            compress_archives: true,
        }
    }
}

pub struct ServerLogger {
    log_file_path: PathBuf,
    config: LogRotationConfig, // NEW: Store config instance
    should_log_requests: bool, // NEW: Configurable logging flags
    should_log_security: bool,
    should_log_performance: bool,
}

impl ServerLogger {
    // NEW: Primary constructor with full config
    pub fn new_with_config(
        server_name: &str,
        port: u16,
        logging_config: &LoggingConfig,
    ) -> Result<Self> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let log_file_path = base_dir
            .join(".rss")
            .join("servers")
            .join(format!("{}-[{}].log", server_name, port));

        if let Some(parent) = log_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        let config = LogRotationConfig::from_main_config(logging_config);

        Ok(Self {
            log_file_path,
            config,
            should_log_requests: logging_config.log_requests,
            should_log_security: logging_config.log_security_alerts,
            should_log_performance: logging_config.log_performance,
        })
    }

    // Legacy constructor for backward compatibility
    pub fn new(server_name: &str, port: u16) -> Result<Self> {
        let default_config = LoggingConfig::default();
        Self::new_with_config(server_name, port, &default_config)
    }

    pub async fn log_server_start(&self) -> Result<()> {
        let entry = self.create_system_entry(LogEventType::ServerStart);
        self.write_log_entry(entry).await
    }

    pub async fn log_server_stop(&self) -> Result<()> {
        let entry = self.create_system_entry(LogEventType::ServerStop);
        self.write_log_entry(entry).await
    }

    fn create_system_entry(&self, event_type: LogEventType) -> ServerLogEntry {
        ServerLogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type,
            ip_address: "127.0.0.1".to_string(),
            user_agent: None,
            method: "SYSTEM".to_string(),
            path: "/".to_string(),
            status_code: None,
            response_time_ms: None,
            bytes_sent: None,
            referer: None,
            query_string: None,
            headers: std::collections::HashMap::new(),
            session_id: None,
        }
    }

    // UPDATED: Check if request logging is enabled
    pub async fn log_request(
        &self,
        req: &actix_web::HttpRequest,
        status: u16,
        response_time: u64,
        bytes_sent: u64,
    ) -> Result<()> {
        if !self.should_log_requests {
            return Ok(()); // Skip if disabled in config
        }

        let ip = {
            let connection_info = req.connection_info();
            connection_info
                .realip_remote_addr()
                .or_else(|| connection_info.peer_addr())
                .unwrap_or("unknown")
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string()
        };

        let headers: std::collections::HashMap<String, String> = req
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                let header_name = name.as_str().to_lowercase();
                if !["authorization", "cookie", "x-api-key"].contains(&header_name.as_str()) {
                    value
                        .to_str()
                        .ok()
                        .map(|v| (name.as_str().to_string(), v.to_string()))
                } else {
                    Some((name.as_str().to_string(), "[FILTERED]".to_string()))
                }
            })
            .collect();

        let entry = ServerLogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: LogEventType::Request,
            ip_address: ip,
            user_agent: req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            method: req.method().to_string(),
            path: req.path().to_string(),
            status_code: Some(status),
            response_time_ms: Some(response_time),
            bytes_sent: Some(bytes_sent),
            referer: req
                .headers()
                .get("referer")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            query_string: if req.query_string().is_empty() {
                None
            } else {
                Some(req.query_string().to_string())
            },
            headers,
            session_id: req.extensions().get::<String>().cloned(),
        };

        self.write_log_entry(entry).await
    }

    // UPDATED: Check if security logging is enabled
    pub async fn log_security_alert(&self, ip: &str, reason: &str, details: &str) -> Result<()> {
        if !self.should_log_security {
            return Ok(()); // Skip if disabled in config
        }

        let mut headers = std::collections::HashMap::new();
        headers.insert("alert_reason".to_string(), reason.to_string());
        headers.insert("alert_details".to_string(), details.to_string());

        let entry = ServerLogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: LogEventType::SecurityAlert,
            ip_address: ip.to_string(),
            user_agent: None,
            method: "SECURITY".to_string(),
            path: "/".to_string(),
            status_code: None,
            response_time_ms: None,
            bytes_sent: None,
            referer: None,
            query_string: None,
            headers,
            session_id: None,
        };

        self.write_log_entry(entry).await
    }

    // NEW: Performance logging
    pub async fn log_performance_warning(
        &self,
        metric: &str,
        value: u64,
        threshold: u64,
    ) -> Result<()> {
        if !self.should_log_performance {
            return Ok(()); // Skip if disabled in config
        }

        let mut headers = std::collections::HashMap::new();
        headers.insert("metric".to_string(), metric.to_string());
        headers.insert("value".to_string(), value.to_string());
        headers.insert("threshold".to_string(), threshold.to_string());

        let entry = ServerLogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: LogEventType::PerformanceWarning,
            ip_address: "127.0.0.1".to_string(),
            user_agent: None,
            method: "PERFORMANCE".to_string(),
            path: "/".to_string(),
            status_code: None,
            response_time_ms: Some(value),
            bytes_sent: None,
            referer: None,
            query_string: None,
            headers,
            session_id: None,
        };

        self.write_log_entry(entry).await
    }

    pub async fn write_log_entry(&self, entry: ServerLogEntry) -> Result<()> {
        self.check_and_rotate_if_needed().await?;

        let json_line = serde_json::to_string(&entry)
            .map_err(|e| AppError::Validation(format!("Failed to serialize log entry: {}", e)))?;

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .await
            .map_err(AppError::Io)?;

        tokio::io::AsyncWriteExt::write_all(&mut file, format!("{}\n", json_line).as_bytes())
            .await
            .map_err(AppError::Io)?;

        tokio::io::AsyncWriteExt::flush(&mut file)
            .await
            .map_err(AppError::Io)?;
        Ok(())
    }

    // UPDATED: Use instance config instead of default
    async fn check_and_rotate_if_needed(&self) -> Result<()> {
        if !self.log_file_path.exists() {
            return Ok(());
        }

        let metadata = tokio::fs::metadata(&self.log_file_path)
            .await
            .map_err(AppError::Io)?;
        if metadata.len() >= self.config.max_file_size_bytes {
            self.rotate_log_files().await?;
        }

        Ok(())
    }

    // UPDATED: Use instance config
    async fn rotate_log_files(&self) -> Result<()> {
        let base_path = &self.log_file_path;
        let base_name = base_path.file_stem().unwrap().to_string_lossy();
        let parent_dir = base_path.parent().unwrap();

        // Move existing archives backward
        for i in (1..self.config.max_archive_files).rev() {
            let old_gz_path = parent_dir.join(format!("{}.{}.log.gz", base_name, i));
            let old_log_path = parent_dir.join(format!("{}.{}.log", base_name, i));
            let new_gz_path = parent_dir.join(format!("{}.{}.log.gz", base_name, i + 1));
            let new_log_path = parent_dir.join(format!("{}.{}.log", base_name, i + 1));

            if old_gz_path.exists() {
                tokio::fs::rename(&old_gz_path, &new_gz_path)
                    .await
                    .map_err(AppError::Io)?;
            } else if old_log_path.exists() {
                tokio::fs::rename(&old_log_path, &new_log_path)
                    .await
                    .map_err(AppError::Io)?;
            }
        }

        // Move current file to .1
        let archive_path = parent_dir.join(format!("{}.1.log", base_name));
        tokio::fs::rename(base_path, &archive_path)
            .await
            .map_err(AppError::Io)?;

        // Compression
        if self.config.compress_archives {
            self.compress_log_file(&archive_path).await?;
        }

        // Cleanup
        let cleanup_num = self.config.max_archive_files + 1;
        let cleanup_log = parent_dir.join(format!("{}.{}.log", base_name, cleanup_num));
        let cleanup_gz = parent_dir.join(format!("{}.{}.log.gz", base_name, cleanup_num));

        if cleanup_log.exists() {
            tokio::fs::remove_file(&cleanup_log)
                .await
                .map_err(AppError::Io)?;
        }
        if cleanup_gz.exists() {
            tokio::fs::remove_file(&cleanup_gz)
                .await
                .map_err(AppError::Io)?;
        }

        Ok(())
    }

    async fn compress_log_file(&self, file_path: &std::path::Path) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let content = tokio::fs::read(file_path).await.map_err(AppError::Io)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content).map_err(AppError::Io)?;
        let compressed = encoder.finish().map_err(AppError::Io)?;

        let gz_path = file_path.with_extension("log.gz");
        tokio::fs::write(&gz_path, compressed)
            .await
            .map_err(AppError::Io)?;
        tokio::fs::remove_file(file_path)
            .await
            .map_err(AppError::Io)?;

        Ok(())
    }

    pub fn get_log_file_size_bytes(&self) -> Result<u64> {
        if !self.log_file_path.exists() {
            return Ok(0);
        }
        let metadata = std::fs::metadata(&self.log_file_path).map_err(AppError::Io)?;
        Ok(metadata.len())
    }

    pub fn list_log_files(&self) -> Result<Vec<PathBuf>> {
        let parent_dir = self.log_file_path.parent().unwrap();
        let base_name = self.log_file_path.file_stem().unwrap().to_string_lossy();
        let mut files = Vec::new();

        if self.log_file_path.exists() {
            files.push(self.log_file_path.clone());
        }

        for i in 1..=10 {
            let archive_path = parent_dir.join(format!("{}.{}.log", base_name, i));
            let gz_path = parent_dir.join(format!("{}.{}.log.gz", base_name, i));

            if archive_path.exists() {
                files.push(archive_path);
            } else if gz_path.exists() {
                files.push(gz_path);
            }
        }

        Ok(files)
    }

    pub async fn get_request_stats(&self) -> Result<ServerStats> {
        if !self.log_file_path.exists() {
            return Ok(ServerStats::default());
        }

        let content = tokio::fs::read_to_string(&self.log_file_path)
            .await
            .map_err(AppError::Io)?;
        let mut stats = ServerStats::default();
        let mut unique_ips = std::collections::HashSet::new();
        let mut response_times = Vec::new();

        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<ServerLogEntry>(line) {
                match entry.event_type {
                    LogEventType::Request => {
                        stats.total_requests += 1;
                        unique_ips.insert(entry.ip_address.clone());

                        if let Some(status) = entry.status_code {
                            if status >= 400 {
                                stats.error_requests += 1;
                            }
                        }

                        if let Some(response_time) = entry.response_time_ms {
                            response_times.push(response_time);
                        }

                        if let Some(bytes) = entry.bytes_sent {
                            stats.total_bytes_sent += bytes;
                        }
                    }
                    LogEventType::SecurityAlert => {
                        stats.security_alerts += 1;
                    }
                    LogEventType::PerformanceWarning => {
                        stats.performance_warnings += 1;
                    }
                    _ => {}
                }
            }
        }

        stats.unique_ips = unique_ips.len() as u64;

        if !response_times.is_empty() {
            response_times.sort();
            stats.avg_response_time =
                response_times.iter().sum::<u64>() / response_times.len() as u64;
            stats.max_response_time = *response_times.last().unwrap_or(&0);
        }

        Ok(stats)
    }

    // NEW: Getters for config validation
    pub fn get_config_summary(&self) -> String {
        format!(
            "Log Config: Max Size {}MB, Archives: {}, Compression: {}, Requests: {}, Security: {}, Performance: {}",
            self.config.max_file_size_bytes / 1024 / 1024,
            self.config.max_archive_files,
            self.config.compress_archives,
            self.should_log_requests,
            self.should_log_security,
            self.should_log_performance
        )
    }
}

#[derive(Debug, Default)]
pub struct ServerStats {
    pub total_requests: u64,
    pub unique_ips: u64,
    pub error_requests: u64,
    pub security_alerts: u64,
    pub performance_warnings: u64, // NEW: Track performance warnings
    pub total_bytes_sent: u64,
    pub avg_response_time: u64,
    pub max_response_time: u64,
}
