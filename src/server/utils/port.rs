use crate::core::config::Config;
use crate::core::prelude::*;

pub enum PortStatus {
    Available,
    OccupiedByUs,
    OccupiedByOther,
}

pub fn check_port_status(port: u16) -> PortStatus {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    // frei?
    if let Ok(listener) = TcpListener::bind(addr) {
        drop(listener);
        return PortStatus::Available;
    }

    // belegt – prüfen ob von uns
    let ctx = crate::server::shared::get_shared_context();
    if let Ok(servers) = ctx.servers.read() {
        if servers.values().any(|s| s.port == port) {
            return PortStatus::OccupiedByUs;
        }
    }

    PortStatus::OccupiedByOther
}

pub fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port))
        .map(|l| {
            drop(l);
            true
        })
        .unwrap_or(false)
}

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

pub fn find_next_available_port_legacy() -> Result<u16> {
    let ctx = crate::server::shared::get_shared_context();
    let servers = ctx.servers.read().unwrap();
    let mut used_ports: Vec<u16> = servers.values().map(|s| s.port).collect();
    used_ports.sort();

    let mut candidate_port = 8080;

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
