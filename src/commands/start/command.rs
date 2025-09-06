// Enhanced src/commands/start/command.rs - RANGE & BULK SUPPORT
use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::port::is_port_available;
use crate::server::utils::validation::find_server;
use opener;

#[derive(Debug, Default)]
pub struct StartCommand;

impl StartCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for StartCommand {
    fn name(&self) -> &'static str {
        "start"
    }
    fn description(&self) -> &'static str {
        "Start server(s) - supports ranges and bulk operations"
    }
    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("start")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        if args.is_empty() {
            return Err(AppError::Validation(
                "Server-ID/Name fehlt! Verwende 'start <ID>', 'start 1-3', 'start all'".to_string(),
            ));
        }

        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        match self.parse_start_args(args) {
            StartMode::Single(identifier) => self.start_single_server(&config, ctx, &identifier),
            StartMode::Range(start, end) => self.start_range_servers(&config, ctx, start, end),
            StartMode::All => self.start_all_servers(&config, ctx),
            StartMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        66
    }
}

#[derive(Debug)]
enum StartMode {
    Single(String),
    Range(u32, u32),
    All,
    Invalid(String),
}

impl StartCommand {
    // Parse different start argument patterns
    fn parse_start_args(&self, args: &[&str]) -> StartMode {
        if args.len() != 1 {
            return StartMode::Invalid("Too many arguments".to_string());
        }

        let arg = args[0];

        // "start all"
        if arg.eq_ignore_ascii_case("all") {
            return StartMode::All;
        }

        // "start 1-3" or "start 001-005"
        if let Some((start_str, end_str)) = arg.split_once('-') {
            match (start_str.parse::<u32>(), end_str.parse::<u32>()) {
                (Ok(start), Ok(end)) => {
                    if start == 0 || end == 0 {
                        return StartMode::Invalid("Range indices must be > 0".to_string());
                    }
                    if start > end {
                        return StartMode::Invalid("Start must be <= end in range".to_string());
                    }
                    if end - start > 20 {
                        return StartMode::Invalid(
                            "Maximum 20 servers in range operation".to_string(),
                        );
                    }
                    StartMode::Range(start, end)
                }
                _ => StartMode::Single(arg.to_string()),
            }
        } else {
            // Single server by ID/name/number
            StartMode::Single(arg.to_string())
        }
    }

    // Start single server (existing robust logic)
    fn start_single_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
    ) -> Result<String> {
        let (server_info, existing_handle) =
            {
                let servers_guard = ctx.servers.read().map_err(|_| {
                    AppError::Validation("Server-Context lock poisoned".to_string())
                })?;

                let server_info = find_server(&servers_guard, identifier)?.clone();

                if server_info.status == ServerStatus::Running {
                    let handles_guard = ctx.handles.read().map_err(|_| {
                        AppError::Validation("Handle-Context lock poisoned".to_string())
                    })?;

                    if handles_guard.contains_key(&server_info.id) {
                        return Ok(format!(
                            "Server '{}' is already running on http://127.0.0.1:{}",
                            server_info.name, server_info.port
                        ));
                    }
                }

                let handles_guard = ctx.handles.read().map_err(|_| {
                    AppError::Validation("Handle-Context lock poisoned".to_string())
                })?;
                let existing_handle = handles_guard.get(&server_info.id).cloned();

                (server_info, existing_handle)
            };

        if let Some(_handle) = existing_handle {
            if server_info.status != ServerStatus::Running {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);

                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Running)
                        .await;
                });
            }

            return Ok(format!(
                "Server '{}' is already running on http://127.0.0.1:{} (status corrected)",
                server_info.name, server_info.port
            ));
        }

        // Port validation
        match self.validate_port_safely(&server_info) {
            PortValidationResult::Available => {}
            PortValidationResult::OccupiedByUs => {
                return Ok(format!(
                    "Port {} wird bereits von unserem System verwendet",
                    server_info.port
                ));
            }
            PortValidationResult::OccupiedByOther => {
                return Ok(format!(
                    "Port {} ist von anderem Prozess belegt! Server '{}' bleibt gestoppt.",
                    server_info.port, server_info.name
                ));
            }
        }

        // Server limit check
        let running_count = self.count_running_servers(ctx);
        if running_count >= config.server.max_concurrent {
            return Err(AppError::Validation(format!(
                "Cannot start server: Running servers limit reached ({}/{})",
                running_count, config.server.max_concurrent
            )));
        }

        self.actually_start_server(config, ctx, server_info, running_count)
    }

    // Start servers by range (e.g., "start 1-3")
    fn start_range_servers(
        &self,
        config: &Config,
        ctx: &ServerContext,
        start: u32,
        end: u32,
    ) -> Result<String> {
        let mut results = Vec::new();
        let mut started_count = 0;
        let mut failed_count = 0;

        for i in start..=end {
            let identifier = format!("{}", i);

            match self.start_single_server(config, ctx, &identifier) {
                Ok(message) => {
                    if message.contains("successfully started") {
                        started_count += 1;
                        results.push(format!("Server {}: Started", i));
                    } else {
                        results.push(format!("Server {}: {}", i, message));
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(format!("Server {}: Failed - {}", i, e));

                    // Stop on critical errors, continue on limit/port issues
                    if e.to_string().contains("limit reached") {
                        break;
                    }
                }
            }
        }

        let summary = format!(
            "Range start completed: {} started, {} failed (Range: {}-{})",
            started_count, failed_count, start, end
        );

        if results.is_empty() {
            Ok(summary)
        } else {
            Ok(format!("{}\n\nResults:\n{}", summary, results.join("\n")))
        }
    }

    // Start all stopped servers
    fn start_all_servers(&self, config: &Config, ctx: &ServerContext) -> Result<String> {
        let stopped_servers: Vec<_> = {
            let servers = ctx.servers.read().unwrap();
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Stopped)
                .map(|s| (s.id.clone(), s.name.clone()))
                .collect()
        };

        if stopped_servers.is_empty() {
            return Ok("No stopped servers to start".to_string());
        }

        if stopped_servers.len() > 20 {
            return Err(AppError::Validation(
                "Too many servers to start at once (max 20). Use ranges instead.".to_string(),
            ));
        }

        let mut results = Vec::new();
        let mut started_count = 0;
        let mut failed_count = 0;

        for (server_id, server_name) in stopped_servers {
            match self.start_single_server(config, ctx, &server_id) {
                Ok(message) => {
                    if message.contains("successfully started") {
                        started_count += 1;
                        results.push(format!("{}: Started", server_name));
                    } else {
                        results.push(format!("{}: {}", server_name, message));
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(format!("{}: Failed - {}", server_name, e));

                    if e.to_string().contains("limit reached") {
                        break;
                    }
                }
            }
        }

        let summary = format!(
            "Start all completed: {} started, {} failed",
            started_count, failed_count
        );

        Ok(format!("{}\n\nResults:\n{}", summary, results.join("\n")))
    }

    // Actually start the server (extracted from single server logic)
    fn actually_start_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
        current_running_count: usize,
    ) -> Result<String> {
        match self.spawn_server(config, ctx, server_info.clone()) {
            Ok(handle) => {
                {
                    let mut handles = ctx.handles.write().unwrap();
                    handles.insert(server_info.id.clone(), handle);
                }

                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);

                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Running)
                        .await;
                });

                let server_url = format!("http://127.0.0.1:{}", server_info.port);

                if config.server.auto_open_browser {
                    self.spawn_browser_opener(server_url.clone(), server_info.name.clone(), config);
                }

                Ok(format!(
                    "Server '{}' successfully started on {} [PERSISTENT] ({}/{} running){}",
                    server_info.name,
                    server_url,
                    current_running_count + 1,
                    config.server.max_concurrent,
                    if config.server.auto_open_browser {
                        " - Browser opening..."
                    } else {
                        ""
                    }
                ))
            }
            Err(e) => {
                self.update_server_status(ctx, &server_info.id, ServerStatus::Failed);

                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Failed)
                        .await;
                });

                Err(AppError::Validation(format!("Server start failed: {}", e)))
            }
        }
    }

    // Helper methods (unchanged from robust version)
    fn validate_port_safely(
        &self,
        server_info: &crate::server::types::ServerInfo,
    ) -> PortValidationResult {
        if is_port_available(server_info.port) {
            PortValidationResult::Available
        } else {
            match crate::server::utils::port::check_port_status(server_info.port) {
                crate::server::utils::port::PortStatus::Available => {
                    PortValidationResult::Available
                }
                crate::server::utils::port::PortStatus::OccupiedByUs => {
                    PortValidationResult::OccupiedByUs
                }
                crate::server::utils::port::PortStatus::OccupiedByOther => {
                    PortValidationResult::OccupiedByOther
                }
            }
        }
    }

    fn count_running_servers(&self, ctx: &ServerContext) -> usize {
        let servers = ctx.servers.read().unwrap();
        servers
            .values()
            .filter(|s| s.status == ServerStatus::Running)
            .count()
    }

    fn spawn_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
    ) -> std::result::Result<actix_web::dev::ServerHandle, String> {
        crate::server::handlers::web::create_web_server(ctx, server_info, config)
    }

    fn spawn_browser_opener(&self, url: String, name: String, config: &Config) {
        let delay = config.server.startup_delay_ms.min(2000);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
            if let Err(e) = opener::open(&url) {
                log::warn!("Failed to open browser for '{}': {}", name, e);
            }
        });
    }

    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}

#[derive(Debug)]
enum PortValidationResult {
    Available,
    OccupiedByUs,
    OccupiedByOther,
}
