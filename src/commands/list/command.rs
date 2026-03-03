use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::server::types::{ServerContext, ServerStatus};

#[derive(Debug, Default)]
pub struct ListCommand;

impl ListCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ListCommand {
    fn name(&self) -> &'static str {
        "list"
    }
    fn description(&self) -> &'static str {
        "List all web servers (persistent)"
    }
    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd == "list" || cmd == "list servers" || cmd == "list server"
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let config = get_config()?;
        let ctx = crate::server::shared::get_shared_context();

        let opts = Self::parse_args(args);

        // Special mode: list memory
        if opts.show_memory {
            return Ok(self.list_memory(ctx, &config));
        }

        Ok(self.list_servers(ctx, &config, opts.status_filter, opts.sort_mode))
    }

    fn priority(&self) -> u8 {
        60
    }
}

#[derive(Debug, Clone, Copy)]
enum SortMode {
    PortAsc,
    PortDesc,
    NameAsc,
    NameDesc,
}

struct ListOpts {
    status_filter: Option<ServerStatus>,
    sort_mode: SortMode,
    show_memory: bool,
}

impl ListCommand {
    /// Parse args: status filter + sort flags + special modes
    fn parse_args(args: &[&str]) -> ListOpts {
        let mut status_filter = None;
        let mut sort_mode = SortMode::PortAsc;
        let mut show_memory = false;

        let mut i = 0;
        while i < args.len() {
            let arg = args[i].to_lowercase();
            match arg.as_str() {
                "running" => status_filter = Some(ServerStatus::Running),
                "stopped" => status_filter = Some(ServerStatus::Stopped),
                "failed" => status_filter = Some(ServerStatus::Failed),
                "memory" | "mem" => show_memory = true,
                "-port" | "--port" => {
                    let dir = args.get(i + 1).map(|s| s.to_lowercase());
                    sort_mode = if dir.as_deref() == Some("desc") {
                        i += 1;
                        SortMode::PortDesc
                    } else {
                        if dir.as_deref() == Some("asc") { i += 1; }
                        SortMode::PortAsc
                    };
                }
                "-name" | "--name" => {
                    let dir = args.get(i + 1).map(|s| s.to_lowercase());
                    sort_mode = if dir.as_deref() == Some("desc") {
                        i += 1;
                        SortMode::NameDesc
                    } else {
                        if dir.as_deref() == Some("asc") { i += 1; }
                        SortMode::NameAsc
                    };
                }
                _ => {}
            }
            i += 1;
        }

        ListOpts { status_filter, sort_mode, show_memory }
    }

    fn list_servers(
        &self,
        ctx: &ServerContext,
        config: &Config,
        status_filter: Option<ServerStatus>,
        sort_mode: SortMode,
    ) -> String {
        let servers = match ctx.servers.read() {
            Ok(s) => s,
            Err(e) => {
                log::error!("servers lock poisoned: {}", e);
                return "Error: server lock poisoned".to_string();
            }
        };

        if servers.is_empty() {
            return "No servers created. Use 'create' to add one.".to_string();
        }

        let mut server_list: Vec<_> = servers.values().collect();

        // Filter
        if let Some(filter) = status_filter {
            server_list.retain(|s| s.status == filter);
        }

        if server_list.is_empty() {
            let filter_name = match status_filter {
                Some(ServerStatus::Running) => "running",
                Some(ServerStatus::Stopped) => "stopped",
                Some(ServerStatus::Failed) => "failed",
                None => "matching",
            };
            return format!("No {} servers found.", filter_name);
        }

        // Sort
        match sort_mode {
            SortMode::PortAsc => server_list.sort_by_key(|s| s.port),
            SortMode::PortDesc => server_list.sort_by(|a, b| b.port.cmp(&a.port)),
            SortMode::NameAsc => server_list.sort_by(|a, b| a.name.cmp(&b.name)),
            SortMode::NameDesc => server_list.sort_by(|a, b| b.name.cmp(&a.name)),
        }

        let running = servers
            .values()
            .filter(|s| s.status == ServerStatus::Running)
            .count();
        let total = servers.len();

        let filter_label = match status_filter {
            Some(ServerStatus::Running) => " [Running]",
            Some(ServerStatus::Stopped) => " [Stopped]",
            Some(ServerStatus::Failed) => " [Failed]",
            None => "",
        };

        let mut result = format!(
            "\n  Servers ({}/{} running, max {}){}\n\n",
            running, total, config.server.max_concurrent, filter_label
        );

        for (i, server) in server_list.iter().enumerate() {
            let status = match server.status {
                ServerStatus::Running => "[Running]",
                ServerStatus::Stopped => "[Stopped]",
                ServerStatus::Failed => "[Failed]",
            };

            let url = format!(
                "http://{}:{}",
                config.server.bind_address, server.port
            );

            result.push_str(&format!(
                "  {:>3}. {:<12} {}  {}\n",
                i + 1,
                server.name,
                url,
                status,
            ));
        }

        result
    }

    /// Show memory/disk usage per server directory + process total
    fn list_memory(&self, ctx: &ServerContext, _config: &Config) -> String {
        let servers = match ctx.servers.read() {
            Ok(s) => s,
            Err(e) => {
                log::error!("servers lock poisoned: {}", e);
                return "Error: server lock poisoned".to_string();
            }
        };

        if servers.is_empty() {
            return "No servers created.".to_string();
        }

        let mut server_list: Vec<_> = servers.values().collect();
        server_list.sort_by_key(|s| s.port);

        let base_dir = crate::core::helpers::get_base_dir().ok();

        // Collect sizes
        let mut entries: Vec<(String, u16, String, u64)> = Vec::new();
        let mut total_disk: u64 = 0;

        for server in &server_list {
            let dir_size = base_dir.as_ref().map_or(0, |base| {
                let dir = base.join("www").join(format!("{}-[{}]", server.name, server.port));
                Self::dir_size(&dir)
            });
            total_disk += dir_size;

            let status = match server.status {
                ServerStatus::Running => "[Running]",
                ServerStatus::Stopped => "[Stopped]",
                ServerStatus::Failed => "[Failed]",
            };

            entries.push((server.name.clone(), server.port, status.to_string(), dir_size));
        }

        // Sort by size descending
        entries.sort_by(|a, b| b.3.cmp(&a.3));

        let process_mem = Self::get_process_memory();

        let mut result = format!(
            "\n  Memory & Disk Usage ({} servers)\n\n",
            server_list.len()
        );

        for (i, (name, port, status, size)) in entries.iter().enumerate() {
            result.push_str(&format!(
                "  {:>3}. {:<12} :{:<5} {:>8}  {}\n",
                i + 1,
                name,
                port,
                Self::format_bytes(*size),
                status,
            ));
        }

        result.push_str(&format!(
            "\n  Total disk: {}",
            Self::format_bytes(total_disk)
        ));

        if !process_mem.is_empty() {
            result.push_str(&format!("  |  Process RAM: {}", process_mem));
        }

        result.push('\n');
        result
    }

    /// Calculate directory size recursively
    fn dir_size(path: &std::path::Path) -> u64 {
        if !path.exists() {
            return 0;
        }
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if meta.is_dir() {
                    total += Self::dir_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
        total
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes == 0 {
            return "0 B".to_string();
        }
        let units = ["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;
        while size >= 1024.0 && unit_idx < units.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        if unit_idx == 0 {
            format!("{} B", bytes)
        } else {
            format!("{:.1} {}", size, units[unit_idx])
        }
    }

    /// Get process RSS memory
    fn get_process_memory() -> String {
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
                format!("{:.1} MB", rss_mb)
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
                        return format!("{:.1} MB", kb / 1024.0);
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
}
