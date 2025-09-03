// ===== src/server/handlers/web/templates.rs =====
use super::PROXY_PORT;
use crate::server::types::ServerData;
use actix_web::{web, HttpResponse, Result as ActixResult};

pub async fn serve_system_dashboard(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let template = include_str!("../templates/rss/dashboard.html");

    let html_content = template
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string())
        .replace("{{PROXY_PORT}}", &PROXY_PORT.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = crate::server::handlers::web::server::inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}
