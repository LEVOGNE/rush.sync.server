use super::ServerDataWithConfig;
use crate::server::logging::ServerLogger;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn logs_raw_handler(
    req: HttpRequest,
    data: web::Data<ServerDataWithConfig>,
) -> ActixResult<HttpResponse> {
    let exe_path = std::env::current_exe().unwrap();
    let base_dir = exe_path.parent().unwrap();
    let log_file_path = base_dir
        .join(".rss")
        .join("servers")
        .join(format!("{}-[{}].log", data.server.name, data.server.port));

    if !log_file_path.exists() {
        return Ok(HttpResponse::Ok().json(json!({
            "new_entries": [],
            "file_size": 0,
            "total_lines": 0,
            "status": "no_log_file"
        })));
    }

    let metadata = match fs::metadata(&log_file_path).await {
        Ok(meta) => meta,
        Err(e) => {
            log::error!("Failed to read log file metadata: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to access log file"
            })));
        }
    };

    let current_file_size = metadata.len();

    let last_known_size: u64 = req
        .headers()
        .get("X-Log-Size")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if current_file_size <= last_known_size {
        return Ok(HttpResponse::Ok().json(json!({
            "new_entries": [],
            "file_size": current_file_size,
            "total_lines": 0,
            "status": "no_new_data"
        })));
    }

    let new_entries = match read_log_entries_from_offset(&log_file_path, last_known_size).await {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("Failed to read log entries: {}", e);
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to read log entries"
            })));
        }
    };

    let stats = get_log_stats(&log_file_path).await.ok();

    Ok(HttpResponse::Ok().json(json!({
        "new_entries": new_entries,
        "file_size": current_file_size,
        "total_lines": new_entries.len(),
        "status": "success",
        "stats": stats
    })))
}

pub async fn logs_handler(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let server_dir = format!("www/{}-[{}]", data.server.name, data.server.port);
    let log_path = format!(
        ".rss/servers/{}-[{}].log",
        data.server.name, data.server.port
    );

    let log_entries = if let Ok(logger) = ServerLogger::new(&data.server.name, data.server.port) {
        match logger.get_log_file_size_bytes() {
            Ok(size) if size > 0 => format!("Log file size: {} bytes", size),
            _ => "No log entries yet".to_string(),
        }
    } else {
        "Logger unavailable".to_string()
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="de">
<head>
   <meta charset="UTF-8">
   <meta name="viewport" content="width=device-width, initial-scale=1.0">
   <title>Server Logs - {}</title>
   <link rel="icon" href="/.rss/favicon.svg" type="image/svg+xml">
   <style>
       body {{
           font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', 'Monaco', monospace;
           margin: 0;
           background: #1a1d23;
           color: #ffffff;
           padding: 1rem;
       }}
       .header {{
           background: #252830;
           padding: 1rem;
           border-radius: 6px;
           border: 1px solid #3a3f47;
           margin-bottom: 1rem;
       }}
       .header h1 {{
           margin: 0 0 0.75rem 0;
           font-size: 18px;
           color: #ffffff;
       }}
       .server-info {{
           font-size: 11px;
           color: #a0a6b1;
           line-height: 1.5;
       }}
       .back-link {{
           color: #00d4ff;
           text-decoration: none;
           font-size: 11px;
       }}
       .back-link:hover {{
           background: #00d4ff;
           color: #1a1d23;
           padding: 2px 4px;
           border-radius: 3px;
       }}
       .hot-reload-status {{
           color: #00ff88;
           font-weight: bold;
       }}
       .log-container {{
           background: #0d1117;
           border: 1px solid #3a3f47;
           border-radius: 6px;
           padding: 1rem;
           max-height: 600px;
           overflow-y: auto;
       }}
       .log-entry {{
           margin: 2px 0;
           font-size: 11px;
           color: #58a6ff;
           line-height: 1.3;
       }}
   </style>
   <script>setInterval(function() {{ location.reload(); }}, 5000);</script>
</head>
<body>
   <div class="header">
       <h1>Server Logs: {}</h1>
       <div class="server-info">
           <p>ID: {} | HTTP: {} | Proxy: {}.localhost:{}</p>
           <p>Directory: {} | Log: {}</p>
           <p class="hot-reload-status">Hot Reload: ACTIVE (WebSocket on /ws/hot-reload)</p>
           <p><a href="/" class="back-link">← Zurück zur Hauptseite</a></p>
       </div>
   </div>
   <div class="log-container">
       <div class="log-entry">Server Directory: {}</div>
       <div class="log-entry">HTTP: http://127.0.0.1:{}</div>
       <div class="log-entry">Proxy: https://{}.localhost:{}</div>
       <div class="log-entry">TLS Certificate: .rss/certs/{}-{}.cert</div>
       <div class="log-entry">Log Status: {}</div>
       <div class="log-entry">Static Files: Enabled (Template-based)</div>
       <div class="log-entry">Hot Reload: WebSocket active on /ws/hot-reload</div>
       <div class="log-entry">File Watcher: Monitoring www directory for changes</div>
       <div class="log-entry">Configuration: Loaded from rush.toml</div>
       <div class="log-entry">--- REAL LOG ENTRIES WOULD APPEAR HERE ---</div>
       <div class="log-entry">Live logging with rotation, security alerts, and performance monitoring</div>
   </div>
</body>
</html>"#,
        data.server.name,
        data.server.name,
        data.server.id,
        data.server.port,
        data.server.name,
        data.proxy_https_port, // FIXED: Verwende proxy_https_port aus data
        server_dir,
        log_path,
        server_dir,
        data.server.port,
        data.server.name,
        data.proxy_https_port, // FIXED: Verwende proxy_https_port aus data
        data.server.name,
        data.server.port,
        log_entries
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn read_log_entries_from_offset(
    file_path: &PathBuf,
    offset: u64,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let file = fs::File::open(file_path).await?;
    let mut reader = BufReader::new(file);

    if offset > 0 {
        use tokio::io::AsyncSeekExt;
        let mut file_with_seek = fs::File::open(file_path).await?;
        file_with_seek
            .seek(std::io::SeekFrom::Start(offset))
            .await?;
        reader = BufReader::new(file_with_seek);
    }

    let mut entries = Vec::new();
    let mut line = String::new();
    let mut lines_read = 0;
    const MAX_LINES_PER_REQUEST: usize = 100;

    while lines_read < MAX_LINES_PER_REQUEST {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            break;
        }

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        if let Ok(json_entry) = serde_json::from_str::<serde_json::Value>(trimmed_line) {
            entries.push(json_entry);
        } else {
            entries.push(json!({
                "timestamp": chrono::Local::now().to_rfc3339(),
                "timestamp_unix": chrono::Utc::now().timestamp(),
                "event_type": "PlainText",
                "message": trimmed_line,
                "level": "INFO"
            }));
        }

        lines_read += 1;
    }

    Ok(entries)
}

async fn get_log_stats(
    file_path: &PathBuf,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let file = fs::File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut total_requests = 0;
    let mut error_requests = 0;
    let mut unique_ips = std::collections::HashSet::new();
    let mut total_bytes = 0u64;
    let mut response_times = Vec::new();

    let mut line_count = 0;
    const MAX_LINES_FOR_STATS: usize = 1000;

    while let Ok(Some(line)) = lines.next_line().await {
        if line_count >= MAX_LINES_FOR_STATS {
            break;
        }

        if let Ok(log_entry) = serde_json::from_str::<serde_json::Value>(&line) {
            if log_entry.get("event_type").and_then(|v| v.as_str()) == Some("Request") {
                total_requests += 1;

                if let Some(ip) = log_entry.get("ip_address").and_then(|v| v.as_str()) {
                    unique_ips.insert(ip.to_string());
                }

                if let Some(status) = log_entry.get("status_code").and_then(|v| v.as_u64()) {
                    if status >= 400 {
                        error_requests += 1;
                    }
                }

                if let Some(bytes) = log_entry.get("bytes_sent").and_then(|v| v.as_u64()) {
                    total_bytes += bytes;
                }

                if let Some(rt) = log_entry.get("response_time_ms").and_then(|v| v.as_u64()) {
                    response_times.push(rt);
                }
            }
        }

        line_count += 1;
    }

    let avg_response_time = if response_times.is_empty() {
        0
    } else {
        response_times.iter().sum::<u64>() / response_times.len() as u64
    };

    let max_response_time = response_times.iter().max().copied().unwrap_or(0);

    Ok(json!({
        "total_requests": total_requests,
        "error_requests": error_requests,
        "unique_ips": unique_ips.len(),
        "total_bytes_sent": total_bytes,
        "avg_response_time_ms": avg_response_time,
        "max_response_time_ms": max_response_time,
        "lines_processed": line_count
    }))
}
