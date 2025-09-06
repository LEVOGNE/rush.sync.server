// src/server/redirect.rs
use crate::core::prelude::*;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

/// HTTP→HTTPS Redirect Server - Teil des Server-Systems
pub struct HttpRedirectServer {
    port: u16,
    target_port: u16,
}

impl HttpRedirectServer {
    pub fn new(port: u16, target_port: u16) -> Self {
        Self { port, target_port }
    }

    async fn redirect_handler(req: HttpRequest, target_port: web::Data<u16>) -> HttpResponse {
        let host = req
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");

        // Host ohne Port
        let host_clean = host.split(':').next().unwrap_or(host);

        let path = req.uri().path();
        let query = req.uri().query().unwrap_or("");

        // Redirect URL bauen
        let redirect_url = if *target_port.get_ref() == 443 {
            format!("https://{}{}", host_clean, path)
        } else {
            format!("https://{}:{}{}", host_clean, target_port.get_ref(), path)
        };

        let final_url = if !query.is_empty() {
            format!("{}?{}", redirect_url, query)
        } else {
            redirect_url
        };

        log::debug!("HTTP→HTTPS: {} → {}", req.uri(), final_url);

        HttpResponse::MovedPermanently()
            .insert_header(("Location", final_url))
            .insert_header(("Strict-Transport-Security", "max-age=31536000"))
            .finish()
    }

    pub async fn run(self) -> Result<()> {
        log::info!("HTTP redirect server starting on port {}", self.port);
        log::info!("Redirecting to HTTPS port {}", self.target_port);

        let target_port = self.target_port;

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(target_port))
                .default_service(web::route().to(Self::redirect_handler))
        })
        .bind(("0.0.0.0", self.port))
        .map_err(|e| AppError::Validation(format!("Port {} bind failed: {}", self.port, e)))?
        .run()
        .await
        .map_err(|e| AppError::Io(e))
    }
}
