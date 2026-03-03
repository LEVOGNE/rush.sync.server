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

        // Extract --workers N from args
        let (filtered_args, workers_override) = Self::extract_workers_flag(args);

        if filtered_args.is_empty() {
            return Err(AppError::Validation(get_translation(
                "server.error.id_missing",
                &[],
            )));
        }

        let filtered_refs: Vec<&str> = filtered_args.iter().map(|s| s.as_str()).collect();

        match parse_bulk_args(&filtered_refs) {
            BulkMode::Single(identifier) => {
                self.start_server_internal(&config, ctx, &identifier, false, workers_override)
            }
            BulkMode::Range(start, end) => {
                self.start_range_servers(&config, ctx, start, end, workers_override)
            }
            BulkMode::All => self.start_all_servers(&config, ctx, workers_override),
            BulkMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        66
    }
}

impl StartCommand {
    /// Extract --workers N flag from args, return remaining args + workers value
    fn extract_workers_flag(args: &[&str]) -> (Vec<String>, Option<usize>) {
        let mut filtered = Vec::new();
        let mut workers = None;
        let mut skip_next = false;

        for (i, arg) in args.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if *arg == "--workers" || *arg == "-w" {
                if let Some(next) = args.get(i + 1) {
                    if let Ok(w) = next.parse::<usize>() {
                        if w >= 1 && w <= 16 {
                            workers = Some(w);
                            skip_next = true;
                            continue;
                        }
                    }
                }
                // Invalid --workers value, keep it as regular arg
                filtered.push(arg.to_string());
            } else {
                filtered.push(arg.to_string());
            }
        }

        (filtered, workers)
    }

    // Internal start logic
    fn start_server_internal(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
        skip_browser: bool,
        workers_override: Option<usize>,
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

        self.actually_start_server(
            config,
            ctx,
            server_info,
            running_count,
            skip_browser,
            workers_override,
        )
    }

    /// Parallel batch size for bulk operations.
    /// Each batch starts N servers concurrently, overlapping the startup delay.
    /// Kept conservative to avoid FD exhaustion with many servers.
    const PARALLEL_BATCH_SIZE: usize = 4;

    // Start servers by range — NON-BLOCKING, PARALLEL with progress
    fn start_range_servers(
        &self,
        config: &Config,
        _ctx: &ServerContext,
        start: u32,
        end: u32,
        workers_override: Option<usize>,
    ) -> Result<String> {
        let total = (end - start + 1) as usize;
        let config = config.clone();
        let rt_handle = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            let _guard = rt_handle.enter();
            let ctx = crate::server::shared::get_shared_context();
            let timer = std::time::Instant::now();

            let identifiers: Vec<_> = (start..=end)
                .map(|i| (format!("{}", i), i))
                .collect();

            let (started, failed) = Self::start_batch_parallel(
                &config, ctx, &identifiers, total, workers_override, &rt_handle,
            );

            let elapsed = timer.elapsed();
            let mem_info = Self::get_memory_info();
            crate::input::send_progress(format!(
                "\n  Range {}-{}: {} [Started], {} [Failed]\n  Time: {:.2}s{}\n",
                start, end, started, failed, elapsed.as_secs_f64(), mem_info,
            ));
        });

        Ok(format!("  Starting {} servers (range {}-{})...", total, start, end))
    }

    // Start all stopped servers — NON-BLOCKING, PARALLEL with progress
    fn start_all_servers(
        &self,
        config: &Config,
        ctx: &ServerContext,
        workers_override: Option<usize>,
    ) -> Result<String> {
        let mut stopped_servers: Vec<_> = {
            let servers = read_lock(&ctx.servers, "servers")?;
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Stopped)
                .map(|s| (s.id.clone(), s.name.clone(), s.port))
                .collect()
        };

        if stopped_servers.is_empty() {
            return Ok("No stopped servers to start".to_string());
        }

        stopped_servers.sort_by_key(|(_, _, port)| *port);

        let total = stopped_servers.len();
        let config = config.clone();
        let rt_handle = tokio::runtime::Handle::current();

        // Prepare identifiers with index for progress
        let servers_with_idx: Vec<_> = stopped_servers
            .iter()
            .enumerate()
            .map(|(i, (id, name, port))| (id.clone(), name.clone(), *port, i as u32))
            .collect();

        std::thread::spawn(move || {
            let _guard = rt_handle.enter();
            let ctx = crate::server::shared::get_shared_context();
            let timer = std::time::Instant::now();

            let identifiers: Vec<_> = servers_with_idx
                .iter()
                .map(|(id, _name, _port, idx)| (id.clone(), *idx))
                .collect();

            // Map for port lookup
            let port_map: std::collections::HashMap<String, (String, u16)> = servers_with_idx
                .iter()
                .map(|(id, name, port, _)| (id.clone(), (name.clone(), *port)))
                .collect();

            let (started, failed) = Self::start_batch_parallel_with_names(
                &config, ctx, &identifiers, &port_map, total, workers_override, &rt_handle,
            );

            let elapsed = timer.elapsed();
            let mem_info = Self::get_memory_info();
            crate::input::send_progress(format!(
                "\n  Done: {} [Started], {} [Failed] (of {})\n  Time: {:.2}s{}\n",
                started, failed, total, elapsed.as_secs_f64(), mem_info,
            ));
        });

        Ok(format!("  Starting {} servers ({} parallel)...", total, Self::PARALLEL_BATCH_SIZE))
    }

    /// Start servers in parallel batches (for range operations)
    fn start_batch_parallel(
        config: &Config,
        ctx: &ServerContext,
        identifiers: &[(String, u32)],
        total: usize,
        workers_override: Option<usize>,
        rt_handle: &tokio::runtime::Handle,
    ) -> (usize, usize) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let started = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let progress_idx = AtomicUsize::new(0);

        for chunk in identifiers.chunks(Self::PARALLEL_BATCH_SIZE) {
            std::thread::scope(|s| {
                for (identifier, num) in chunk {
                    let idx = progress_idx.fetch_add(1, Ordering::Relaxed) + 1;
                    crate::input::send_progress(format!(
                        "  [{}/{}] Starting server {}...",
                        idx, total, num
                    ));

                    let started = &started;
                    let failed = &failed;
                    let rt = rt_handle.clone();

                    s.spawn(move || {
                        let _g = rt.enter();
                        let cmd = StartCommand::new();
                        match cmd.start_server_internal(config, ctx, identifier, true, workers_override) {
                            Ok(message) => {
                                if message.contains("started successfully") {
                                    started.fetch_add(1, Ordering::Relaxed);
                                    let port = Self::extract_port_from_message(&message);
                                    let name = Self::extract_name_from_message(&message)
                                        .unwrap_or_else(|| format!("server-{}", num));
                                    let url_str = port
                                        .map(|p| format!("  http://127.0.0.1:{}", p))
                                        .unwrap_or_default();
                                    crate::input::send_progress(format!(
                                        "  {}: [Started]{}",
                                        name, url_str
                                    ));
                                } else {
                                    crate::input::send_progress(format!("  Server {}: {}", num, message));
                                }
                            }
                            Err(e) => {
                                failed.fetch_add(1, Ordering::Relaxed);
                                crate::input::send_progress(format!(
                                    "  Server {}: [Failed] - {}",
                                    num, e
                                ));
                            }
                        }
                    });
                }
            });
        }

        (started.load(Ordering::Relaxed), failed.load(Ordering::Relaxed))
    }

    /// Start servers in parallel batches (for "all" operations with name/port info)
    fn start_batch_parallel_with_names(
        config: &Config,
        ctx: &ServerContext,
        identifiers: &[(String, u32)],
        port_map: &std::collections::HashMap<String, (String, u16)>,
        total: usize,
        workers_override: Option<usize>,
        rt_handle: &tokio::runtime::Handle,
    ) -> (usize, usize) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let started = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let progress_idx = AtomicUsize::new(0);

        for chunk in identifiers.chunks(Self::PARALLEL_BATCH_SIZE) {
            std::thread::scope(|s| {
                for (server_id, _idx) in chunk {
                    let idx = progress_idx.fetch_add(1, Ordering::Relaxed) + 1;
                    let (name, port) = port_map
                        .get(server_id)
                        .cloned()
                        .unwrap_or_else(|| (server_id.clone(), 0));

                    crate::input::send_progress(format!(
                        "  [{}/{}] Starting {}...",
                        idx, total, name
                    ));

                    let started = &started;
                    let failed = &failed;
                    let rt = rt_handle.clone();

                    s.spawn(move || {
                        let _g = rt.enter();
                        let cmd = StartCommand::new();
                        match cmd.start_server_internal(config, ctx, server_id, true, workers_override) {
                            Ok(message) => {
                                if message.contains("started successfully") {
                                    started.fetch_add(1, Ordering::Relaxed);
                                    crate::input::send_progress(format!(
                                        "  {}: [Started]  http://127.0.0.1:{}",
                                        name, port
                                    ));
                                } else {
                                    crate::input::send_progress(format!("  {}: {}", name, message));
                                }
                            }
                            Err(e) => {
                                failed.fetch_add(1, Ordering::Relaxed);
                                crate::input::send_progress(format!(
                                    "  {}: [Failed] - {}",
                                    name, e
                                ));
                            }
                        }
                    });
                }
            });
        }

        (started.load(Ordering::Relaxed), failed.load(Ordering::Relaxed))
    }

    // Actually start the server
    fn actually_start_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        server_info: crate::server::types::ServerInfo,
        current_running_count: usize,
        skip_browser: bool,
        workers_override: Option<usize>,
    ) -> Result<String> {
        match self.spawn_server(config, ctx, server_info.clone(), workers_override) {
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
                let proxy_http_port = config.proxy.port;
                let proxy_https_port = config.proxy.port + config.proxy.https_port_offset;
                let actual_workers = workers_override.unwrap_or(config.server.workers);

                let open_browser = !skip_browser && config.server.auto_open_browser;
                if open_browser {
                    self.spawn_browser_opener(server_url.clone(), server_info.name.clone(), config);
                }

                Ok(format!(
                    "\n  Server '{}' started successfully [PERSISTENT]\n\n  \
                     HTTP        {}\n  \
                     Proxy HTTP  http://{}.localhost:{}\n  \
                     Proxy HTTPS https://{}.localhost:{}\n  \
                     Dashboard   {}/.rss/\n  \
                     Directory   www/{}-[{}]/\n  \
                     Workers     {}\n\n  \
                     Running {}/{}{}\n",
                    server_info.name,
                    server_url,
                    server_info.name, proxy_http_port,
                    server_info.name, proxy_https_port,
                    server_url,
                    server_info.name, server_info.port,
                    actual_workers,
                    current_running_count + 1,
                    config.server.max_concurrent,
                    if open_browser {
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

    /// Get process memory usage for benchmarking
    fn get_memory_info() -> String {
        #[cfg(target_os = "macos")]
        {
            use std::mem;
            extern "C" {
                fn mach_task_self() -> u32;
                fn task_info(
                    task: u32,
                    flavor: u32,
                    info: *mut libc::c_void,
                    count: *mut u32,
                ) -> i32;
            }

            #[repr(C)]
            struct MachTaskBasicInfo {
                virtual_size: u64,
                resident_size: u64,
                resident_size_max: u64,
                user_time: [u32; 2],
                system_time: [u32; 2],
                policy: i32,
                suspend_count: i32,
            }

            let mut info: MachTaskBasicInfo = unsafe { mem::zeroed() };
            let mut count = (mem::size_of::<MachTaskBasicInfo>() / mem::size_of::<u32>()) as u32;

            let result = unsafe {
                task_info(
                    mach_task_self(),
                    20, // MACH_TASK_BASIC_INFO
                    &mut info as *mut _ as *mut libc::c_void,
                    &mut count,
                )
            };

            if result == 0 {
                let rss_mb = info.resident_size as f64 / (1024.0 * 1024.0);
                format!("  |  Memory: {:.1} MB", rss_mb)
            } else {
                String::new()
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let kb: f64 = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        return format!("  |  Memory: {:.1} MB", kb / 1024.0);
                    }
                }
            }
            String::new()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            String::new()
        }
    }

    // Helper methods
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
        workers_override: Option<usize>,
    ) -> std::result::Result<actix_web::dev::ServerHandle, String> {
        crate::server::handlers::web::create_web_server_with_workers(
            ctx,
            server_info,
            config,
            workers_override,
        )
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

    /// Extract port from a success message like "http://127.0.0.1:8001"
    fn extract_port_from_message(message: &str) -> Option<u16> {
        message
            .find("127.0.0.1:")
            .and_then(|pos| {
                let after = &message[pos + 10..];
                let port_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                port_str.parse().ok()
            })
    }

    /// Extract server name from a success message like "Server 'rss-001' started"
    fn extract_name_from_message(message: &str) -> Option<String> {
        let start = message.find('\'')?;
        let end = message[start + 1..].find('\'')?;
        Some(message[start + 1..start + 1 + end].to_string())
    }
}

#[derive(Debug)]
enum PortValidationResult {
    Available,
    OccupiedByUs,
    OccupiedByOther,
}
