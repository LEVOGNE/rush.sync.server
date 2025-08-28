// Fixed src/server/utils/port.rs
use crate::core::config::Config;
use crate::core::prelude::*;
use std::net::TcpListener;
use std::time::Duration;

pub fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener);
            std::thread::sleep(Duration::from_millis(10));
            TcpListener::bind(("127.0.0.1", port)).is_ok()
        }
        Err(_) => false,
    }
}

// Updated to use Config instead of ServerContext
pub fn find_next_available_port(config: &Config) -> Result<u16> {
    let ctx = crate::server::shared::get_shared_context();
    let servers = ctx.servers.read().unwrap();
    let mut used_ports: Vec<u16> = servers.values().map(|s| s.port).collect();
    used_ports.sort();

    let mut candidate_port = config.server.port_range_start;
    let max_port = config.server.port_range_end;

    loop {
        if candidate_port > max_port {
            return Err(AppError::Validation(format!(
                "No available ports in range {}-{}",
                config.server.port_range_start, config.server.port_range_end
            )));
        }

        if !used_ports.contains(&candidate_port) && is_port_available(candidate_port) {
            return Ok(candidate_port);
        }

        candidate_port += 1;
    }
}

// Legacy function for backward compatibility - REMOVED ServerContext dependency
pub fn find_next_available_port_legacy() -> Result<u16> {
    // This function is deprecated and only provides basic functionality
    let ctx = crate::server::shared::get_shared_context();
    let servers = ctx.servers.read().unwrap();
    let mut used_ports: Vec<u16> = servers.values().map(|s| s.port).collect();
    used_ports.sort();

    let mut candidate_port = 8080; // Fallback default

    loop {
        if !used_ports.contains(&candidate_port) && is_port_available(candidate_port) {
            return Ok(candidate_port);
        }

        candidate_port += 1;
        if candidate_port > 8180 {
            return Err(AppError::Validation(
                "No available ports found in default range 8080-8180".to_string(),
            ));
        }
    }
}
