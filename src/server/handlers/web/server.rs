// ===== src/server/handlers/web/server.rs =====
use super::PROXY_PORT;
use crate::server::types::ServerData;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};

pub async fn serve_fallback_or_inject(
    req: HttpRequest,
    data: web::Data<ServerData>,
) -> ActixResult<HttpResponse> {
    let path = req.path();
    log::info!("Requested path: {}", path);

    let exe_path = std::env::current_exe().unwrap();
    let base_dir = exe_path.parent().unwrap();
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.name, data.port));

    let file_path = if path == "/" {
        server_dir.join("index.html")
    } else {
        server_dir.join(path.trim_start_matches('/'))
    };

    log::info!("Looking for file: {:?}", file_path);

    if file_path.exists() {
        if let Some(extension) = file_path.extension() {
            if extension == "html" {
                log::info!("Loading custom HTML file");
                match tokio::fs::read_to_string(&file_path).await {
                    Ok(mut html_content) => {
                        if !html_content.contains("/rss.js") {
                            html_content = inject_rss_script(html_content);
                        }

                        return Ok(HttpResponse::Ok()
                            .content_type("text/html; charset=utf-8")
                            .body(html_content));
                    }
                    Err(e) => {
                        log::error!("Failed to read HTML file: {}", e);
                    }
                }
            } else {
                log::info!("Serving static file: {:?}", file_path);
                match tokio::fs::read(&file_path).await {
                    Ok(content) => {
                        let content_type = match extension.to_str() {
                            Some("css") => "text/css",
                            Some("js") => "application/javascript",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("svg") => "image/svg+xml",
                            _ => "application/octet-stream",
                        };

                        return Ok(HttpResponse::Ok().content_type(content_type).body(content));
                    }
                    Err(e) => {
                        log::error!("Failed to read file: {}", e);
                    }
                }
            }
        }
    }

    if path == "/" {
        log::info!("Serving system fallback");
        serve_system_fallback(data).await
    } else {
        log::info!("File not found: {}", path);
        Ok(HttpResponse::NotFound()
            .content_type("text/plain")
            .body("File not found"))
    }
}

async fn serve_system_fallback(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let template = include_str!("../templates/rss/dashboard.html");

    let html_content = template
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string())
        .replace("{{PROXY_PORT}}", &PROXY_PORT.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}

pub fn inject_rss_script(html: String) -> String {
    let script_tag = r#"<script src="/rss.js"></script>"#;
    let css_link = r#"<link rel="stylesheet" href="/.rss/global-reset.css">"#;

    let html_with_css = if let Some(head_end) = html.find("</head>") {
        let (before, after) = html.split_at(head_end);
        format!("{}\n    {}\n{}", before, css_link, after)
    } else {
        format!("{}\n{}", css_link, html)
    };

    if let Some(body_end) = html_with_css.rfind("</body>") {
        let (before, after) = html_with_css.split_at(body_end);
        format!("{}\n    {}\n{}", before, script_tag, after)
    } else if let Some(html_end) = html_with_css.rfind("</html>") {
        let (before, after) = html_with_css.split_at(html_end);
        format!("{}\n{}\n{}", before, script_tag, after)
    } else {
        format!("{}\n{}", html_with_css, script_tag)
    }
}
