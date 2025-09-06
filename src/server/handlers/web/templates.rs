use super::ServerDataWithConfig;
use actix_web::{web, HttpResponse, Result as ActixResult};

pub async fn serve_system_dashboard(
    data: web::Data<ServerDataWithConfig>,
) -> ActixResult<HttpResponse> {
    let template = include_str!("../templates/rss/dashboard.html");

    // KORREKTE Template-Ersetzung:
    let html_content = template
        .replace("{{SERVER_NAME}}", &data.server.name)
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string()) // HTTP = 3000
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string()) // HTTPS = 3443
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = crate::server::handlers::web::server::inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}
