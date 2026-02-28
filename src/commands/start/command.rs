use crate::commands::command::Command;
use crate::commands::parsing::{parse_bulk_args, BulkMode};
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
            return Err(AppError::Validation(get_translation(
                "server.error.id_missing",
                &[],
            )));
        }

        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        match parse_bulk_args(args) {
            BulkMode::Single(identifier) => self.start_single_server(&config, ctx, &identifier),
            BulkMode::Range(start, end) => self.start_range_servers(&config, ctx, start, end),
            BulkMode::All => self.start_all_servers(&config, ctx),
            BulkMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        66
    }
}

impl StartCommand {
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
                            "Server '{}' is already running on http://{}:{}",
                            server_info.name, config.server.bind_address, server_info.port
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
                "Server '{}' is already running on http://{}:{} (status corrected)",
                server_info.name, config.server.bind_address, server_info.port
            ));
        }

        // Port validation
        match self.validate_port_safely(&server_info, &config.server.bind_address) {
            PortValidationResult::Available => {}
            PortValidationResult::OccupiedByUs => {
                return Ok(get_translation(
                    "server.error.port_used_by_us",
                    &[&server_info.port.to_string()],
                ));
            }
            PortValidationResult::OccupiedByOther => {
                return Ok(get_translation(
                    "server.error.port_used_by_other",
                    &[&server_info.port.to_string(), &server_info.name],
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
            let servers = read_lock(&ctx.servers, "servers")?;
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Stopped)
                .map(|s| (s.id.clone(), s.name.clone()))
                .collect()
        };

        if stopped_servers.is_empty() {
            return Ok("No stopped servers to start".to_string());
        }

        if stopped_servers.len() > 50 {
            log::warn!(
                "Starting {} servers - this may take a while",
                stopped_servers.len()
            );
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
                    let mut handles = write_lock(&ctx.handles, "handles")?;
                    handles.insert(server_info.id.clone(), handle);
                }

                self.update_server_status(ctx, &server_info.id, ServerStatus::Running);

                let server_id = server_info.id.clone();
                tokio::spawn(async move {
                    crate::server::shared::persist_server_update(&server_id, ServerStatus::Running)
                        .await;
                });

                let server_url =
                    format!("http://{}:{}", config.server.bind_address, server_info.port);

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
        bind_address: &str,
    ) -> PortValidationResult {
        if is_port_available(server_info.port, bind_address) {
            PortValidationResult::Available
        } else {
            match crate::server::utils::port::check_port_status(server_info.port, bind_address) {
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
        let servers = match ctx.servers.read() {
            Ok(s) => s,
            Err(e) => {
                log::error!("servers lock poisoned: {}", e);
                return 0;
            }
        };
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
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
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
