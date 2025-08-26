use crate::core::prelude::*;
use crate::server::types::ServerContext;
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

pub fn find_next_available_port(ctx: &ServerContext) -> Result<u16> {
    let servers = ctx.servers.read().unwrap();

    let mut used_ports: Vec<u16> = servers.values().map(|s| s.port).collect();
    used_ports.sort();

    let mut candidate_port = ctx.port_range_start;

    loop {
        if !used_ports.contains(&candidate_port) && is_port_available(candidate_port) {
            return Ok(candidate_port);
        }

        candidate_port += 1;

        if candidate_port > ctx.port_range_start + 100 {
            return Err(AppError::Validation(
                "Keine verfügbaren Ports gefunden".to_string(),
            ));
        }
    }
}
