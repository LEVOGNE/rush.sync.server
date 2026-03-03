use crate::commands::command::Command;
use crate::commands::parsing::{parse_bulk_args, BulkMode};
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};
use crate::server::utils::validation::find_server;

#[derive(Debug, Default)]
pub struct StopCommand;

impl StopCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for StopCommand {
    fn name(&self) -> &'static str {
        "stop"
    }

    fn description(&self) -> &'static str {
        "Stop server(s) - supports ranges and bulk operations"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("stop")
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
            BulkMode::Single(identifier) => self.stop_single_server(&config, ctx, &identifier, false),
            BulkMode::Range(start, end) => self.stop_range_servers(&config, ctx, start, end),
            BulkMode::All => self.stop_all_servers(&config, ctx),
            BulkMode::Invalid(error) => Err(AppError::Validation(error)),
        }
    }

    fn priority(&self) -> u8 {
        67
    }
}

impl StopCommand {
    /// Parallel batch size for bulk stop operations
    const PARALLEL_BATCH_SIZE: usize = 4;

    // Stop single server
    // `bulk_mode`: when true, skip the blocking sleep (for parallel bulk ops)
    fn stop_single_server(
        &self,
        config: &Config,
        ctx: &ServerContext,
        identifier: &str,
        bulk_mode: bool,
    ) -> Result<String> {
        let (server_info, handle) = {
            let servers_guard = ctx
                .servers
                .read()
                .map_err(|_| AppError::Validation("Server-Context lock poisoned".to_string()))?;

            let server_info = find_server(&servers_guard, identifier)?.clone();

            if server_info.status != ServerStatus::Running {
                return Ok(format!(
                    "Server '{}' is not active (Status: {})",
                    server_info.name, server_info.status
                ));
            }

            // Atomically remove the handle
            let handle = {
                let mut handles_guard = ctx.handles.write().map_err(|_| {
                    AppError::Validation("Handle-Context lock poisoned".to_string())
                })?;
                handles_guard.remove(&server_info.id)
            };

            (server_info, handle)
        };

        log::info!(
            "Stopping server {} on port {}",
            server_info.id,
            server_info.port
        );

        // Set status to Stopped immediately
        self.update_server_status(ctx, &server_info.id, ServerStatus::Stopped);

        // Notify browser to close (skip in bulk mode for speed)
        if !bulk_mode {
            self.notify_browser_shutdown(&server_info);
        }

        if let Some(handle) = handle {
            // Graceful shutdown (async, non-blocking)
            self.shutdown_server_gracefully(handle, server_info.id.clone(), config);

            // Persist status update (non-blocking)
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            // Only sleep for single-server stop (TUI feedback), skip for bulk
            if !bulk_mode {
                std::thread::sleep(std::time::Duration::from_millis(
                    config.server.startup_delay_ms.min(500),
                ));
            }

            let running_count = {
                let servers = ctx.servers.read().unwrap_or_else(|e| {
                    log::warn!("Server lock poisoned: {}", e);
                    e.into_inner()
                });
                servers
                    .values()
                    .filter(|s| s.status == ServerStatus::Running)
                    .count()
            };

            Ok(format!(
                "Server '{}' stopped [PERSISTENT] ({}/{} running)",
                server_info.name, running_count, config.server.max_concurrent
            ))
        } else {
            // Handle was already removed - just update status
            let server_id = server_info.id.clone();
            tokio::spawn(async move {
                crate::server::shared::persist_server_update(&server_id, ServerStatus::Stopped)
                    .await;
            });

            Ok(format!(
                "Server '{}' was already stopped [PERSISTENT]",
                server_info.name
            ))
        }
    }

    // Stop servers by range — NON-BLOCKING, PARALLEL with progress
    fn stop_range_servers(
        &self,
        config: &Config,
        _ctx: &ServerContext,
        start: u32,
        end: u32,
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

            let (stopped, failed) = Self::stop_batch_parallel(
                &config, ctx, &identifiers, total, &rt_handle,
            );

            let elapsed = timer.elapsed();
            let mem_info = Self::get_memory_info();
            crate::input::send_progress(format!(
                "\n  Range {}-{}: {} [Stopped], {} [Failed]\n  Time: {:.2}s{}\n",
                start, end, stopped, failed, elapsed.as_secs_f64(), mem_info,
            ));
        });

        Ok(format!("  Stopping {} servers (range {}-{}, {} parallel)...", total, start, end, Self::PARALLEL_BATCH_SIZE))
    }

    // Stop all running servers — NON-BLOCKING, PARALLEL with progress, sorted by port
    fn stop_all_servers(&self, config: &Config, ctx: &ServerContext) -> Result<String> {
        let mut running_servers: Vec<_> = {
            let servers = read_lock(&ctx.servers, "servers")?;
            servers
                .values()
                .filter(|s| s.status == ServerStatus::Running)
                .map(|s| (s.id.clone(), s.name.clone(), s.port))
                .collect()
        };

        if running_servers.is_empty() {
            return Ok("No running servers to stop".to_string());
        }

        // Sort by port for consistent ordering
        running_servers.sort_by_key(|(_, _, port)| *port);

        let total = running_servers.len();
        let config = config.clone();
        let rt_handle = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            let _guard = rt_handle.enter();
            let ctx = crate::server::shared::get_shared_context();
            let timer = std::time::Instant::now();

            let identifiers: Vec<_> = running_servers
                .iter()
                .map(|(id, name, _port)| (id.clone(), name.clone()))
                .collect();

            let (stopped, failed) = Self::stop_batch_parallel_with_names(
                &config, ctx, &identifiers, total, &rt_handle,
            );

            let elapsed = timer.elapsed();
            let mem_info = Self::get_memory_info();
            crate::input::send_progress(format!(
                "\n  Done: {} [Stopped], {} [Failed] (of {})\n  Time: {:.2}s{}\n",
                stopped, failed, total, elapsed.as_secs_f64(), mem_info,
            ));
        });

        Ok(format!("  Stopping {} servers ({} parallel)...", total, Self::PARALLEL_BATCH_SIZE))
    }

    /// Stop servers in parallel batches (for range operations)
    fn stop_batch_parallel(
        config: &Config,
        ctx: &ServerContext,
        identifiers: &[(String, u32)],
        total: usize,
        rt_handle: &tokio::runtime::Handle,
    ) -> (usize, usize) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let stopped = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let progress_idx = AtomicUsize::new(0);

        for chunk in identifiers.chunks(Self::PARALLEL_BATCH_SIZE) {
            std::thread::scope(|s| {
                for (identifier, num) in chunk {
                    let idx = progress_idx.fetch_add(1, Ordering::Relaxed) + 1;
                    crate::input::send_progress(format!(
                        "  [{}/{}] Stopping server {}...",
                        idx, total, num
                    ));

                    let stopped = &stopped;
                    let failed = &failed;
                    let rt = rt_handle.clone();

                    s.spawn(move || {
                        let _g = rt.enter();
                        let cmd = StopCommand::new();
                        match cmd.stop_single_server(config, ctx, identifier, true) {
                            Ok(message) => {
                                if message.contains("stopped [PERSISTENT]") {
                                    stopped.fetch_add(1, Ordering::Relaxed);
                                    crate::input::send_progress(format!(
                                        "  Server {}: [Stopped]",
                                        num
                                    ));
                                } else {
                                    crate::input::send_progress(format!(
                                        "  Server {}: {}",
                                        num, message
                                    ));
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

        (stopped.load(Ordering::Relaxed), failed.load(Ordering::Relaxed))
    }

    /// Stop servers in parallel batches (for "all" operations with name info)
    fn stop_batch_parallel_with_names(
        config: &Config,
        ctx: &ServerContext,
        identifiers: &[(String, String)], // (server_id, server_name)
        total: usize,
        rt_handle: &tokio::runtime::Handle,
    ) -> (usize, usize) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let stopped = AtomicUsize::new(0);
        let failed = AtomicUsize::new(0);
        let progress_idx = AtomicUsize::new(0);

        for chunk in identifiers.chunks(Self::PARALLEL_BATCH_SIZE) {
            std::thread::scope(|s| {
                for (server_id, server_name) in chunk {
                    let idx = progress_idx.fetch_add(1, Ordering::Relaxed) + 1;
                    crate::input::send_progress(format!(
                        "  [{}/{}] Stopping {}...",
                        idx, total, server_name
                    ));

                    let stopped = &stopped;
                    let failed = &failed;
                    let rt = rt_handle.clone();

                    s.spawn(move || {
                        let _g = rt.enter();
                        let cmd = StopCommand::new();
                        match cmd.stop_single_server(config, ctx, server_id, true) {
                            Ok(message) => {
                                if message.contains("stopped [PERSISTENT]") {
                                    stopped.fetch_add(1, Ordering::Relaxed);
                                    crate::input::send_progress(format!(
                                        "  {}: [Stopped]",
                                        server_name
                                    ));
                                } else {
                                    crate::input::send_progress(format!(
                                        "  {}: {}",
                                        server_name, message
                                    ));
                                }
                            }
                            Err(e) => {
                                failed.fetch_add(1, Ordering::Relaxed);
                                crate::input::send_progress(format!(
                                    "  {}: [Failed] - {}",
                                    server_name, e
                                ));
                            }
                        }
                    });
                }
            });
        }

        (stopped.load(Ordering::Relaxed), failed.load(Ordering::Relaxed))
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

    // Browser notification
    fn notify_browser_shutdown(&self, server_info: &crate::server::types::ServerInfo) {
        let server_port = server_info.port;
        let server_name = server_info.name.clone();

        tokio::spawn(async move {
            let server_url = format!("http://127.0.0.1:{}", server_port);
            log::info!(
                "Notifying browser to close for server {} (async)",
                server_name
            );

            let client = reqwest::Client::new();
            if let Err(e) = client
                .get(format!("{}/api/close-browser", server_url))
                .timeout(std::time::Duration::from_millis(300))
                .send()
                .await
            {
                log::warn!("Failed to notify browser: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        });
    }

    // Graceful shutdown
    fn shutdown_server_gracefully(
        &self,
        handle: actix_web::dev::ServerHandle,
        server_id: String,
        config: &Config,
    ) {
        let shutdown_timeout = config.server.shutdown_timeout;

        tokio::spawn(async move {
            use tokio::time::{timeout, Duration};

            match timeout(Duration::from_secs(shutdown_timeout), handle.stop(true)).await {
                Ok(_) => log::info!("Server {} stopped gracefully", server_id),
                Err(_) => {
                    log::warn!(
                        "Server {} shutdown timeout ({}s), forcing stop",
                        server_id,
                        shutdown_timeout
                    );
                    handle.stop(false).await;
                }
            }
        });
    }

    // Status update helper
    fn update_server_status(&self, ctx: &ServerContext, server_id: &str, status: ServerStatus) {
        if let Ok(mut servers) = ctx.servers.write() {
            if let Some(server) = servers.get_mut(server_id) {
                server.status = status;
            }
        }
    }
}
