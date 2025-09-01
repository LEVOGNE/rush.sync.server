use crate::core::prelude::*;
use actix::ActorContext;
use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct FileChangeEvent {
    pub event_type: String,
    pub file_path: String,
    pub server_name: String,
    pub port: u16,
    pub timestamp: u64,
    pub file_extension: Option<String>,
}

#[derive(Debug)]
pub struct WatchdogManager {
    watchers: Arc<RwLock<HashMap<String, RecommendedWatcher>>>,
    sender: broadcast::Sender<FileChangeEvent>,
}

impl Default for WatchdogManager {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
            sender,
        }
    }
}

impl WatchdogManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FileChangeEvent> {
        self.sender.subscribe()
    }

    pub fn start_watching(&self, server_name: &str, port: u16) -> Result<()> {
        let exe_path = std::env::current_exe().map_err(AppError::Io)?;
        let base_dir = exe_path.parent().ok_or_else(|| {
            AppError::Validation("Cannot determine executable directory".to_string())
        })?;

        let watch_path = base_dir
            .join("www")
            .join(format!("{}-[{}]", server_name, port));

        if !watch_path.exists() {
            return Err(AppError::Validation(format!(
                "Server directory does not exist: {:?}",
                watch_path
            )));
        }

        let server_key = format!("{}:{}", server_name, port);
        let sender = self.sender.clone();
        let server_name_owned = server_name.to_owned();

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if let Err(e) = handle_file_event(&event, &server_name_owned, port, &sender) {
                        log::error!("Error handling file event: {}", e);
                    }
                }
                Err(e) => log::error!("Watch error: {:?}", e),
            })
            .map_err(|e| AppError::Validation(format!("Failed to create watcher: {}", e)))?;

        watcher
            .watch(&watch_path, RecursiveMode::Recursive)
            .map_err(|e| AppError::Validation(format!("Failed to start watching: {}", e)))?;

        let mut watchers = self.watchers.write().unwrap();
        watchers.insert(server_key.clone(), watcher);

        log::info!(
            "Started file watching for server {} on port {} at {:?}",
            server_name,
            port,
            watch_path
        );
        Ok(())
    }

    pub fn stop_watching(&self, server_name: &str, port: u16) -> Result<()> {
        let server_key = format!("{}:{}", server_name, port);
        let mut watchers = self.watchers.write().unwrap();

        if let Some(_watcher) = watchers.remove(&server_key) {
            log::info!(
                "Stopped file watching for server {} on port {}",
                server_name,
                port
            );
        }

        Ok(())
    }

    pub fn get_active_watchers(&self) -> Vec<String> {
        let watchers = self.watchers.read().unwrap();
        watchers.keys().cloned().collect()
    }
}

fn handle_file_event(
    event: &Event,
    server_name: &str,
    port: u16,
    sender: &broadcast::Sender<FileChangeEvent>,
) -> Result<()> {
    // Nur relevante Events verarbeiten
    let event_type = match event.kind {
        EventKind::Create(_) => "created",
        EventKind::Modify(_) => "modified",
        EventKind::Remove(_) => "deleted",
        _ => return Ok(()), // Ignore andere Events
    };

    for path in &event.paths {
        // Skip temporäre Dateien und Backups
        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            if name.starts_with('.')
                || name.ends_with('~')
                || name.contains(".tmp")
                || name.contains(".swp")
            {
                continue;
            }
        }

        let file_extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        // Nur Web-relevante Dateien
        if let Some(ref ext) = file_extension {
            if ![
                "html", "css", "js", "json", "txt", "md", "svg", "png", "jpg", "jpeg", "gif", "ico",
            ]
            .contains(&ext.as_str())
            {
                continue;
            }
        }

        let change_event = FileChangeEvent {
            event_type: event_type.to_string(),
            file_path: path.to_string_lossy().to_string(),
            server_name: server_name.to_string(),
            port,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            file_extension,
        };

        if let Err(e) = sender.send(change_event) {
            log::error!("Failed to send file change event: {}", e);
        }
    }

    Ok(())
}

// WebSocket Actor für Hot Reload
pub struct HotReloadWs {
    receiver: Option<broadcast::Receiver<FileChangeEvent>>,
    server_filter: Option<String>, // Format: "name:port"
}

impl Actor for HotReloadWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!("WebSocket connection established for hot reload");

        if let Some(mut receiver) = self.receiver.take() {
            let addr = ctx.address();

            tokio::spawn(async move {
                loop {
                    match receiver.recv().await {
                        Ok(event) => {
                            addr.do_send(event);
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            log::warn!("WebSocket lagged, skipped {} events", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            log::debug!("WebSocket event channel closed");
                            break;
                        }
                    }
                }
            });
        }

        // Ping alle 30 Sekunden
        ctx.run_interval(Duration::from_secs(30), |_, ctx| {
            ctx.ping(b"");
        });
    }
}

impl StreamHandler<std::result::Result<ws::Message, ws::ProtocolError>> for HotReloadWs {
    fn handle(
        &mut self,
        msg: std::result::Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => {
                log::debug!("WebSocket received: {}", text);
            }
            Ok(ws::Message::Close(reason)) => {
                log::debug!("WebSocket closing: {:?}", reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<FileChangeEvent> for HotReloadWs {
    type Result = ();

    fn handle(&mut self, msg: FileChangeEvent, ctx: &mut Self::Context) -> Self::Result {
        // Filter nach Server wenn gesetzt
        if let Some(ref filter) = self.server_filter {
            let event_key = format!("{}:{}", msg.server_name, msg.port);
            if *filter != event_key {
                return;
            }
        }

        let json = match serde_json::to_string(&msg) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize file change event: {}", e);
                return;
            }
        };

        ctx.text(json);
    }
}

// WebSocket Endpoint Handler
pub async fn ws_hot_reload(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Arc<WatchdogManager>>,
) -> std::result::Result<HttpResponse, actix_web::Error> {
    let server_filter = req
        .query_string()
        .split('&')
        .find_map(|param| {
            if param.starts_with("server=") {
                param.strip_prefix("server=")
            } else {
                None
            }
        })
        .map(|s| s.to_string());

    let ws_actor = HotReloadWs {
        receiver: Some(data.subscribe()),
        server_filter,
    };

    ws::start(ws_actor, &req, stream)
}

// Static globals für Manager
use std::sync::OnceLock;
static WATCHDOG_MANAGER: OnceLock<Arc<WatchdogManager>> = OnceLock::new();

pub fn get_watchdog_manager() -> &'static Arc<WatchdogManager> {
    WATCHDOG_MANAGER.get_or_init(|| Arc::new(WatchdogManager::new()))
}

pub fn start_server_watching(server_name: &str, port: u16) -> Result<()> {
    get_watchdog_manager().start_watching(server_name, port)
}

pub fn stop_server_watching(server_name: &str, port: u16) -> Result<()> {
    get_watchdog_manager().stop_watching(server_name, port)
}
