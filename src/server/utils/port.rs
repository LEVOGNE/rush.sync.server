use crate::core::config::Config;
use crate::core::prelude::*;

pub enum PortStatus {
    Available,
    OccupiedByUs,
    OccupiedByOther,
}

pub fn check_port_status(port: u16, bind_address: &str) -> PortStatus {
    // free?
    if is_port_available(port, bind_address) {
        return PortStatus::Available;
    }

    // occupied - check if by us
    let ctx = crate::server::shared::get_shared_context();
    if let Ok(servers) = ctx.servers.read() {
        if servers.values().any(|s| s.port == port) {
            return PortStatus::OccupiedByUs;
        }
    }

    PortStatus::OccupiedByOther
}

pub fn is_port_available(port: u16, bind_address: &str) -> bool {
    std::net::TcpListener::bind((bind_address, port))
        .map(|l| {
            drop(l);
            true
        })
        .unwrap_or(false)
}

pub fn find_next_available_port(config: &Config) -> Result<u16> {
    let ctx = crate::server::shared::get_shared_context();
    let servers = crate::core::helpers::read_lock(&ctx.servers, "servers")?;
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

        if !used_ports.contains(&candidate_port)
            && is_port_available(candidate_port, &config.server.bind_address)
        {
            return Ok(candidate_port);
        }

        candidate_port += 1;
    }
}
