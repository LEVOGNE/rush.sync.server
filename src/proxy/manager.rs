use crate::core::prelude::*;
use crate::proxy::handler::ProxyServer;
use crate::proxy::types::{ProxyConfig, ProxyRoute, ProxyTarget, RouteMap};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProxyManager {
    config: ProxyConfig,
    routes: Arc<RwLock<RouteMap>>,
    targets: Arc<RwLock<HashMap<String, ProxyTarget>>>,
}

impl ProxyManager {
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            routes: Arc::new(RwLock::new(HashMap::new())),
            targets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_route(&self, server_name: &str, server_id: &str, port: u16) -> Result<()> {
        let route = ProxyRoute {
            subdomain: server_name.to_string(),
            target_port: port,
            server_id: server_id.to_string(),
        };

        let target = ProxyTarget {
            name: server_name.to_string(),
            port,
            healthy: true,
            last_check: std::time::SystemTime::now(),
        };

        {
            let mut routes = self.routes.write().await;
            routes.insert(server_name.to_string(), route);
        }

        {
            let mut targets = self.targets.write().await;
            targets.insert(server_name.to_string(), target);
        }

        log::info!(
            "Added proxy route: {}.localhost -> 127.0.0.1:{}",
            server_name,
            port
        );
        Ok(())
    }

    pub async fn remove_route(&self, server_name: &str) -> Result<()> {
        {
            let mut routes = self.routes.write().await;
            routes.remove(server_name);
        }

        {
            let mut targets = self.targets.write().await;
            targets.remove(server_name);
        }

        log::info!("Removed proxy route: {}.localhost", server_name);
        Ok(())
    }

    pub async fn get_routes(&self) -> Vec<ProxyRoute> {
        let routes = self.routes.read().await;
        routes.values().cloned().collect()
    }

    pub async fn get_target_port(&self, subdomain: &str) -> Option<u16> {
        let routes = self.routes.read().await;
        routes.get(subdomain).map(|route| route.target_port)
    }

    pub fn get_config(&self) -> &ProxyConfig {
        &self.config
    }

    pub async fn start_proxy_server(self: Arc<Self>) -> Result<()> {
        if !self.config.enabled {
            log::info!("Reverse Proxy disabled");
            return Ok(());
        }

        let proxy_server = ProxyServer::new(Arc::clone(&self));

        // HTTPS-Port: immer 8443 (config.port + 443)
        let https_port = 8443;

        log::info!("Starting Reverse Proxy:");
        log::info!("  HTTP:  http://127.0.0.1:{}", self.config.port);
        log::info!("  HTTPS: https://127.0.0.1:{}", https_port);

        tokio::spawn(async move {
            if let Err(e) = proxy_server.start_with_https().await {
                log::error!("Proxy with HTTPS failed: {}", e);
            }
        });

        log::info!(
            "TLS certificate: .rss/certs/proxy-{}.cert",
            self.config.port
        );

        Ok(())
    }
}
