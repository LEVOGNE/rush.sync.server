use crate::core::prelude::*;
use crate::server::types::ServerInfo;
use std::collections::HashMap;

pub fn find_server<'a>(
    servers: &'a HashMap<String, ServerInfo>,
    identifier: &str,
) -> Result<&'a ServerInfo> {
    // ✅ CHRONOLOGISCHE INDEX-SUCHE
    if let Ok(index) = identifier.parse::<usize>() {
        if index > 0 && index <= servers.len() {
            // Sortiere Server chronologisch wie in list_servers
            let mut server_list: Vec<_> = servers.values().collect();
            server_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            return server_list
                .get(index - 1) // Index beginnt bei 0, aber User gibt 1-3 ein
                .copied()
                .ok_or_else(|| AppError::Validation("Server index out of range".to_string()));
        }
    }

    // By name or ID prefix (unverändert)
    for server in servers.values() {
        if server.name == identifier || server.id.starts_with(identifier) {
            return Ok(server);
        }
    }

    Err(AppError::Validation(format!(
        "Server '{}' nicht gefunden",
        identifier
    )))
}

pub fn validate_server_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AppError::Validation(
            "Server name cannot be empty".to_string(),
        ));
    }

    if name.len() > 50 {
        return Err(AppError::Validation("Server name too long".to_string()));
    }

    // Validate characters
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::Validation(
            "Server name can only contain alphanumeric characters, hyphens and underscores"
                .to_string(),
        ));
    }

    Ok(())
}

pub fn validate_port(port: u16) -> Result<()> {
    if port < 1024 {
        return Err(AppError::Validation("Port must be >= 1024".to_string()));
    }

    // u16 kann maximal 65535 sein, also Check entfernt
    Ok(())
}
