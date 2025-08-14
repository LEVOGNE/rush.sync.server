// =====================================================
// FILE: src/server/instance.rs - ACTIX-WEB SERVER INSTANCE
// =====================================================

use crate::core::prelude::*;
use crate::server::middleware;
use crate::server::routes;
use crate::server::{ServerConfig, ServerInfo, ServerMode, ServerStatus};

use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::{Condition, Logger};
use actix_web::{web, App, HttpServer};
use notify::RecommendedWatcher;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Einzelne Server-Instanz mit Actix-Web
pub struct ServerInstance {
    pub info: Arc<Mutex<ServerInfo>>,
    pub config: ServerConfig,
    pub(crate) shutdown_tx: Option<oneshot::Sender<()>>,
    pub(crate) server_handle: Option<actix_web::dev::ServerHandle>, // NEU!
    pub(crate) file_watcher: Option<RecommendedWatcher>,
}

impl ServerInstance {
    /// Erstellt neue Server-Instanz
    pub fn new(port: u16, mode: ServerMode) -> Self {
        let info = ServerInfo::new(port, mode);
        let config = ServerConfig::for_mode(mode, port);

        log::info!(
            "🚀 Creating server instance: {} on port {}",
            info.id[..8].to_uppercase(),
            port
        );

        Self {
            info: Arc::new(Mutex::new(info)),
            config,
            shutdown_tx: None,
            server_handle: None,
            file_watcher: None,
        }
    }

    /// Startet den Actix-Web Server
    pub async fn start(&mut self) -> Result<()> {
        // Status auf Starting setzen
        {
            let mut info = self.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.status = ServerStatus::Starting;
            info.last_modified = Some(chrono::Utc::now());
        }

        // Konfiguration validieren
        self.config.validate()?;

        // Working Directory erstellen
        self.setup_working_directory().await?;

        // File Watcher für Hot-Reloading (nur Dev-Modus)
        if self.config.mode == ServerMode::Dev && self.config.hot_reload {
            self.setup_file_watcher().await?;
        }

        let bind_addr = self.config.bind_address();
        let static_dir = self.config.static_dir.clone();
        let server_info = Arc::clone(&self.info);
        let config = self.config.clone();

        log::info!("🌐 Starting Actix-Web server on {}", bind_addr);

        // Shutdown-Channel erstellen
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Actix-Web Server konfigurieren und starten
        let server = HttpServer::new(move || {
            // CORS-Objekt vorab bauen (gleicher Typ in beiden Zweigen)
            let cors = if config.cors_enabled {
                Cors::permissive()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            } else {
                Cors::default()
            };

            App::new()
                // Logger nur wenn aktiviert (kein Typwechsel dank Condition)
                .wrap(Condition::new(config.debug_logs, Logger::default()))
                // CORS immer wrappen (cors hat in beiden Fällen den gleichen Typ)
                .wrap(cors)
                // Eigene Middleware
                .wrap(middleware::ServerInfoMiddleware::new(Arc::clone(
                    &server_info,
                )))
                // Basis-Routen
                .route("/", web::get().to(routes::index))
                .route("/health", web::get().to(routes::health_check))
                .route("/api/info", web::get().to(routes::server_info))
                .route("/api/status", web::get().to(routes::server_status))
                // Dev-Routen bedingt per configure (verändert App-Typ nicht)
                .configure(|app_cfg| {
                    if config.mode == ServerMode::Dev {
                        app_cfg
                            .route("/dev/reload", web::post().to(routes::dev_reload))
                            .route("/dev/logs", web::get().to(routes::dev_logs));
                    }
                })
                // Statische Dateien zuletzt registrieren
                .service(
                    Files::new("/", static_dir.clone())
                        .index_file("index.html")
                        .use_last_modified(true),
                )
        });

        // Server starten und Handle speichern
        let server = server.run();
        let server_handle = server.handle();
        self.server_handle = Some(server_handle.clone());

        // Server Task mit korrektem Shutdown
        let _server_task = tokio::spawn(async move {
            tokio::select! {
                _ = server => {
                    log::info!("✅ Server stopped");
                }
                _ = shutdown_rx => {
                    log::info!("🛑 Server shutdown requested");
                    server_handle.stop(true).await;
                }
            }
        });

        // Status auf Running setzen
        {
            let mut info = self.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.status = ServerStatus::Running;
            info.last_modified = Some(chrono::Utc::now());
        }

        log::info!(
            "✅ Server {} running on {}",
            self.get_server_id(),
            bind_addr
        );

        Ok(())
    }

    /// Stoppt den Server
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("🛑 Stopping server {}", self.get_server_id());

        // Status auf Stopping setzen
        {
            let mut info = self.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.status = ServerStatus::Stopping;
            info.last_modified = Some(chrono::Utc::now());
        }

        // File Watcher stoppen
        if let Some(watcher) = self.file_watcher.take() {
            drop(watcher);
        }

        // NEU: Server Handle stoppen
        if let Some(handle) = self.server_handle.take() {
            handle.stop(true).await;
        }

        // Shutdown-Signal senden
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        // Kurz warten für graceful shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Status auf Stopped setzen
        {
            let mut info = self.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.status = ServerStatus::Stopped;
            info.last_modified = Some(chrono::Utc::now());
        }

        log::info!("✅ Server {} stopped", self.get_server_id());
        Ok(())
    }

    /// Erstellt Working Directory mit Standard-Dateien
    async fn setup_working_directory(&self) -> Result<()> {
        let working_dir = {
            let info = self.info.lock().unwrap_or_else(|poisoned| {
                log::warn!("Recovered from poisoned mutex");
                poisoned.into_inner()
            });
            info.working_dir.clone()
        };

        // Verzeichnis erstellen
        tokio::fs::create_dir_all(&working_dir)
            .await
            .map_err(AppError::Io)?;

        let static_dir = working_dir.join("static");
        tokio::fs::create_dir_all(&static_dir)
            .await
            .map_err(AppError::Io)?;

        // Standard-Dateien erstellen wenn sie nicht existieren
        self.create_default_files(&static_dir).await?;

        log::debug!("📁 Working directory setup: {}", working_dir.display());
        Ok(())
    }

    /// Erstellt Standard HTML/CSS/JS Dateien
    async fn create_default_files(&self, static_dir: &Path) -> Result<()> {
        let server_id = self.get_server_id();

        // index.html
        let index_path = static_dir.join("index.html");
        if !index_path.exists() {
            let index_content = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rush Sync Server - {}</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>🚀 Rush Sync Server</h1>
        <p>Server ID: <code>{}</code></p>
        <p>Mode: <code>{}</code></p>
        <p>Port: <code>{}</code></p>

        <div class="status">
            <h2>✅ Server Running</h2>
            <p>This server was created by Rush Sync Server v{}</p>
        </div>

        <div class="api-links">
            <h3>API Endpoints:</h3>
            <ul>
                <li><a href="/health">Health Check</a></li>
                <li><a href="/api/info">Server Info</a></li>
                <li><a href="/api/status">Server Status</a></li>
            </ul>
        </div>
    </div>

    <script src="script.js"></script>
</body>
</html>"#,
                server_id,
                server_id,
                self.config.mode,
                self.config.port,
                crate::core::constants::VERSION
            );

            tokio::fs::write(&index_path, index_content)
                .await
                .map_err(AppError::Io)?;
        }

        // style.css
        let css_path = static_dir.join("style.css");
        if !css_path.exists() {
            let css_content = r#"/* Rush Sync Server - Default Styles */
body {
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    margin: 0;
    padding: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: #333;
    min-height: 100vh;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    background: rgba(255, 255, 255, 0.95);
    padding: 40px;
    border-radius: 16px;
    box-shadow: 0 20px 40px rgba(0,0,0,0.1);
}

h1 {
    color: #4a5568;
    text-align: center;
    margin-bottom: 30px;
    font-size: 2.5em;
}

h2 {
    color: #2d3748;
    border-bottom: 2px solid #e2e8f0;
    padding-bottom: 10px;
}

code {
    background: #f7fafc;
    padding: 4px 8px;
    border-radius: 4px;
    font-family: 'Monaco', 'Consolas', monospace;
    color: #e53e3e;
}

.status {
    background: linear-gradient(135deg, #48bb78, #38a169);
    color: white;
    padding: 20px;
    border-radius: 8px;
    margin: 20px 0;
}

.api-links ul {
    list-style: none;
    padding: 0;
}

.api-links li {
    margin: 10px 0;
}

.api-links a {
    display: inline-block;
    padding: 8px 16px;
    background: #4299e1;
    color: white;
    text-decoration: none;
    border-radius: 6px;
    transition: background 0.3s ease;
}

.api-links a:hover {
    background: #3182ce;
}

@media (max-width: 600px) {
    body { padding: 10px; }
    .container { padding: 20px; }
    h1 { font-size: 2em; }
}
"#;

            tokio::fs::write(&css_path, css_content)
                .await
                .map_err(AppError::Io)?;
        }

        // script.js
        let js_path = static_dir.join("script.js");
        if !js_path.exists() {
            let js_content = r#"// Rush Sync Server - Default JavaScript

console.log('🚀 Rush Sync Server loaded!');

// Auto-refresh server status
async function updateStatus() {
    try {
        const response = await fetch('/api/status');
        const status = await response.json();
        console.log('Server status:', status);
    } catch (error) {
        console.error('Failed to fetch status:', error);
    }
}

// Update status every 5 seconds
setInterval(updateStatus, 5000);

// Initial status check
updateStatus();

// Dev-Mode: Auto-reload functionality
if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    console.log('🔧 Dev mode detected - enabling auto-reload');

    // Check for updates every 2 seconds in dev mode
    setInterval(async () => {
        try {
            const response = await fetch('/dev/reload', { method: 'POST' });
            if (response.ok) {
                const result = await response.json();
                if (result.should_reload) {
                    console.log('🔄 Reloading due to file changes...');
                    window.location.reload();
                }
            }
        } catch (error) {
            // Ignore errors in dev mode
        }
    }, 2000);
}
"#;

            tokio::fs::write(&js_path, js_content)
                .await
                .map_err(AppError::Io)?;
        }

        Ok(())
    }

    /// File Watcher für Hot-Reloading einrichten
    async fn setup_file_watcher(&mut self) -> Result<()> {
        // TODO: Implementiere File Watcher mit notify crate
        // Für jetzt erstmal leer lassen - können wir später ausbauen
        log::debug!("🔍 File watcher setup (TODO: implement hot-reloading)");
        Ok(())
    }

    /// Server-ID (ersten 8 Zeichen)
    pub fn get_server_id(&self) -> String {
        let info = self.info.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
        info.id[..8].to_uppercase().to_string()
    }

    /// Server-Status abfragen
    pub fn get_status(&self) -> ServerStatus {
        let info = self.info.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
        info.status.clone()
    }

    /// Debug-Info
    pub fn debug_info(&self) -> String {
        let info = self.info.lock().unwrap_or_else(|poisoned| {
            log::warn!("Recovered from poisoned mutex");
            poisoned.into_inner()
        });
        info.debug_info()
    }
}
