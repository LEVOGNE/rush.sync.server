// ## FILE: src/server/logging.rs - ALLE FIXES APPLIED
use crate::core::config::LoggingConfig;
use crate::core::prelude::*;
use actix_web::HttpMessage; // HINZUGEFÜGT
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub headers: HashMap<String, String>,
    pub session_id: Option<String>,
}

// Copy-Trait hinzugefügt für LogEventType
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

impl From<&LoggingConfig> for LogRotationConfig {
    fn from(config: &LoggingConfig) -> Self {
        Self {
            max_file_size_bytes: config.max_file_size_mb * 1024 * 1024,
            max_archive_files: config.max_archive_files,
            compress_archives: config.compress_archives,
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 100 * 1024 * 1024,
            max_archive_files: 9,
            compress_archives: true,
        }
    }
}

pub struct ServerLogger {
    log_file_path: PathBuf,
    config: LogRotationConfig,
    log_requests: bool,
    log_security: bool,
    log_performance: bool,
}

impl ServerLogger {
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

        Ok(Self {
            log_file_path,
            config: LogRotationConfig::from(logging_config),
            log_requests: logging_config.log_requests,
            log_security: logging_config.log_security_alerts,
            log_performance: logging_config.log_performance,
        })
    }

    pub fn new(server_name: &str, port: u16) -> Result<Self> {
        Self::new_with_config(server_name, port, &LoggingConfig::default())
    }

    // Vereinfachte System-Log Methoden
    pub async fn log_server_start(&self) -> Result<()> {
        self.write_system_entry(LogEventType::ServerStart).await
    }

    pub async fn log_server_stop(&self) -> Result<()> {
        self.write_system_entry(LogEventType::ServerStop).await
    }

    async fn write_system_entry(&self, event_type: LogEventType) -> Result<()> {
        let entry = ServerLogEntry {
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
            headers: HashMap::new(),
            session_id: None,
        };
        self.write_log_entry(entry).await
    }

    // Optimierte Request Logging
    pub async fn log_request(
        &self,
        req: &actix_web::HttpRequest,
        status: u16,
        response_time: u64,
        bytes_sent: u64,
    ) -> Result<()> {
        if !self.log_requests {
            return Ok(());
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

        // Nur relevante Headers filtern
        let headers = req
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                let header_name = name.as_str().to_lowercase();
                match header_name.as_str() {
                    "authorization" | "cookie" | "x-api-key" => {
                        Some((name.as_str().to_string(), "[FILTERED]".to_string()))
                    }
                    _ => value
                        .to_str()
                        .ok()
                        .map(|v| (name.as_str().to_string(), v.to_string())),
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
            ip_address: ip.to_string(),
            user_agent: req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(String::from),
            method: req.method().to_string(),
            path: req.path().to_string(),
            status_code: Some(status),
            response_time_ms: Some(response_time),
            bytes_sent: Some(bytes_sent),
            referer: req
                .headers()
                .get("referer")
                .and_then(|h| h.to_str().ok())
                .map(String::from),
            query_string: if req.query_string().is_empty() {
                None
            } else {
                Some(req.query_string().to_string())
            },
            headers,
            session_id: req.extensions().get::<String>().cloned(), // Jetzt funktioniert es mit HttpMessage import
        };

        self.write_log_entry(entry).await
    }

    // Vereinfachte Alert-Logging
    pub async fn log_security_alert(&self, ip: &str, reason: &str, details: &str) -> Result<()> {
        if !self.log_security {
            return Ok(());
        }
        self.write_alert_entry(LogEventType::SecurityAlert, ip, reason, details, None)
            .await
    }

    pub async fn log_performance_warning(
        &self,
        metric: &str,
        value: u64,
        threshold: u64,
    ) -> Result<()> {
        if !self.log_performance {
            return Ok(());
        }
        self.write_alert_entry(
            LogEventType::PerformanceWarning,
            "127.0.0.1",
            metric,
            &format!("value={}, threshold={}", value, threshold),
            Some(value),
        )
        .await
    }

    // KORRIGIERTE write_alert_entry Method
    async fn write_alert_entry(
        &self,
        event_type: LogEventType,
        ip: &str,
        reason: &str,
        details: &str,
        response_time: Option<u64>,
    ) -> Result<()> {
        let mut headers = HashMap::new();
        headers.insert("alert_reason".to_string(), reason.to_string());
        headers.insert("alert_details".to_string(), details.to_string());

        let method_name = match event_type {
            LogEventType::SecurityAlert => "SECURITY",
            LogEventType::PerformanceWarning => "PERFORMANCE",
            _ => "SYSTEM",
        };

        let entry = ServerLogEntry {
            timestamp: chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string(),
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type, // Jetzt Copy-able, kein Move-Problem mehr
            ip_address: ip.to_string(),
            user_agent: None,
            method: method_name.to_string(),
            path: "/".to_string(),
            status_code: None,
            response_time_ms: response_time,
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

        use tokio::io::AsyncWriteExt;
        file.write_all(format!("{}\n", json_line).as_bytes())
            .await
            .map_err(AppError::Io)?;
        file.flush().await.map_err(AppError::Io)?;
        Ok(())
    }

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

    async fn rotate_log_files(&self) -> Result<()> {
        let base_path = &self.log_file_path;
        let base_name = base_path.file_stem().unwrap().to_string_lossy();
        let parent_dir = base_path.parent().unwrap();

        // Rotate existing archives
        for i in (1..self.config.max_archive_files).rev() {
            let old_path = parent_dir.join(format!("{}.{}.log.gz", base_name, i));
            let new_path = parent_dir.join(format!("{}.{}.log.gz", base_name, i + 1));

            if old_path.exists() {
                tokio::fs::rename(&old_path, &new_path)
                    .await
                    .map_err(AppError::Io)?;
            }
        }

        // Move current log to archive
        let archive_path = parent_dir.join(format!("{}.1.log", base_name));
        tokio::fs::rename(base_path, &archive_path)
            .await
            .map_err(AppError::Io)?;

        if self.config.compress_archives {
            self.compress_log_file(&archive_path).await?;
        }

        // Cleanup old files
        let cleanup_path = parent_dir.join(format!(
            "{}.{}.log.gz",
            base_name,
            self.config.max_archive_files + 1
        ));
        if cleanup_path.exists() {
            tokio::fs::remove_file(&cleanup_path)
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

        let gz_path = file_path.with_file_name(format!(
            "{}.gz",
            file_path
                .file_name()
                .ok_or_else(|| AppError::Validation("Invalid file path".to_string()))?
                .to_string_lossy()
        ));

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
            for ext in &["log", "log.gz"] {
                let path = parent_dir.join(format!("{}.{}.{}", base_name, i, ext));
                if path.exists() {
                    files.push(path);
                    break; // Only add one version (prefer compressed)
                }
            }
        }

        Ok(files)
    }

    // Optimierte Stats-Berechnung
    pub async fn get_request_stats(&self) -> Result<ServerStats> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        if !self.log_file_path.exists() {
            return Ok(ServerStats::default());
        }

        let file = tokio::fs::File::open(&self.log_file_path)
            .await
            .map_err(AppError::Io)?;
        let mut reader = BufReader::new(file).lines();

        let mut stats = ServerStats::default();
        let mut unique_ips = std::collections::HashSet::new();
        let mut response_times = Vec::new();

        while let Some(line) = reader.next_line().await.map_err(AppError::Io)? {
            if let Ok(entry) = serde_json::from_str::<ServerLogEntry>(&line) {
                match entry.event_type {
                    LogEventType::Request => {
                        stats.total_requests += 1;
                        unique_ips.insert(entry.ip_address);

                        if let Some(status) = entry.status_code {
                            if status >= 400 {
                                stats.error_requests += 1;
                            }
                        }
                        if let Some(rt) = entry.response_time_ms {
                            response_times.push(rt);
                        }
                        if let Some(bytes) = entry.bytes_sent {
                            stats.total_bytes_sent += bytes;
                        }
                    }
                    LogEventType::SecurityAlert => stats.security_alerts += 1,
                    LogEventType::PerformanceWarning => stats.performance_warnings += 1,
                    _ => {}
                }
            }
        }

        stats.unique_ips = unique_ips.len() as u64;
        if !response_times.is_empty() {
            stats.avg_response_time =
                response_times.iter().sum::<u64>() / response_times.len() as u64;
            stats.max_response_time = *response_times.iter().max().unwrap_or(&0);
        }

        Ok(stats)
    }

    pub fn get_config_summary(&self) -> String {
        format!(
            "Log Config: {}MB max, {} archives, compression: {}, requests: {}, security: {}, performance: {}",
            self.config.max_file_size_bytes / 1024 / 1024,
            self.config.max_archive_files,
            self.config.compress_archives,
            self.log_requests,
            self.log_security,
            self.log_performance
        )
    }
}

#[derive(Debug, Default)]
pub struct ServerStats {
    pub total_requests: u64,
    pub unique_ips: u64,
    pub error_requests: u64,
    pub security_alerts: u64,
    pub performance_warnings: u64,
    pub total_bytes_sent: u64,
    pub avg_response_time: u64,
    pub max_response_time: u64,
}
