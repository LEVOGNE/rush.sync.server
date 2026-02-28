use super::ServerDataWithConfig;
use crate::core::helpers::html_escape;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};

pub async fn serve_fallback_or_inject(
    req: HttpRequest,
    data: web::Data<ServerDataWithConfig>,
) -> ActixResult<HttpResponse> {
    let path = req.path();
    log::info!("Requested path: {}", path);

    let base_dir = crate::core::helpers::get_base_dir().map_err(|e| {
        log::error!("Failed to get base directory: {}", e);
        actix_web::error::ErrorInternalServerError("Internal server error")
    })?;
    let server_dir = base_dir
        .join("www")
        .join(format!("{}-[{}]", data.server.name, data.server.port));

    let mut file_path = if path == "/" {
        server_dir.join("index.html")
    } else {
        server_dir.join(path.trim_start_matches('/'))
    };

    // Resolve directory paths to index.html
    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    // Path traversal protection: ensure resolved path stays within server_dir
    let canonical_server_dir = server_dir
        .canonicalize()
        .unwrap_or_else(|_| server_dir.clone());
    if let Ok(canonical_file) = file_path.canonicalize() {
        if !canonical_file.starts_with(&canonical_server_dir) {
            log::warn!("Path traversal attempt blocked: {}", path);
            return Ok(HttpResponse::Forbidden()
                .content_type("text/plain")
                .body("Forbidden"));
        }
    }

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
                            Some("jpg" | "jpeg") => "image/jpeg",
                            Some("svg") => "image/svg+xml",
                            Some("gif") => "image/gif",
                            Some("ico") => "image/x-icon",
                            Some("webp") => "image/webp",
                            Some("webm") => "video/webm",
                            Some("mp4") => "video/mp4",
                            Some("woff2") => "font/woff2",
                            Some("woff") => "font/woff",
                            Some("json") => "application/json",
                            Some("xml") => "application/xml",
                            Some("pdf") => "application/pdf",
                            Some("txt" | "md") => "text/plain; charset=utf-8",
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

async fn serve_system_fallback(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let template = include_str!("../templates/rss/dashboard.html");

    let html_content = template
        .replace("{{SERVER_NAME}}", &html_escape(&data.server.name))
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}

pub fn inject_rss_script(html: String) -> String {
    // ES6 module script injection
    let script_tag = r#"<script defer src="/rss.js"></script>"#;
    let css_link = r#"<link rel="stylesheet" href="/.rss/_reset.css">"#;

    // Insert CSS into <head>
    let html_with_css = if let Some(head_end) = html.find("</head>") {
        let (before, after) = html.split_at(head_end);
        format!("{}\n    {}\n{}", before, css_link, after)
    } else {
        format!("{}\n{}", css_link, html)
    };

    // Insert JS module before </body>
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- html_escape tests ---

    #[test]
    fn test_html_escape_basic() {
        assert_eq!(html_escape("hello"), "hello");
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }

    #[test]
    fn test_html_escape_tags() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(html_escape(r#"a"b'c"#), "a&quot;b&#x27;c");
    }

    #[test]
    fn test_html_escape_xss_payload() {
        let payload = r#"<img onerror="alert('xss')" src=x>"#;
        let escaped = html_escape(payload);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('"'));
    }

    #[test]
    fn test_html_escape_empty() {
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_html_escape_safe_names() {
        assert_eq!(html_escape("my-server-01"), "my-server-01");
        assert_eq!(html_escape("test_server"), "test_server");
    }

    // --- inject_rss_script tests ---

    #[test]
    fn test_inject_script_with_head_and_body() {
        let html = "<html><head><title>Test</title></head><body><h1>Hi</h1></body></html>";
        let result = inject_rss_script(html.to_string());
        assert!(result.contains(r#"<link rel="stylesheet" href="/.rss/_reset.css">"#));
        assert!(result.contains(r#"<script type="module" src="/rss.js"></script>"#));
    }

    #[test]
    fn test_inject_script_css_before_head_close() {
        let html = "<html><head></head><body></body></html>";
        let result = inject_rss_script(html.to_string());
        let css_pos = result.find("_reset.css").unwrap();
        let head_end_pos = result.find("</head>").unwrap();
        assert!(css_pos < head_end_pos);
    }

    #[test]
    fn test_inject_script_js_before_body_close() {
        let html = "<html><head></head><body><p>content</p></body></html>";
        let result = inject_rss_script(html.to_string());
        let js_pos = result.find("rss.js").unwrap();
        let body_end_pos = result.find("</body>").unwrap();
        assert!(js_pos < body_end_pos);
    }

    #[test]
    fn test_inject_script_no_body_tag() {
        let html = "<html><head></head></html>";
        let result = inject_rss_script(html.to_string());
        assert!(result.contains("rss.js"));
        let js_pos = result.find("rss.js").unwrap();
        let html_end_pos = result.find("</html>").unwrap();
        assert!(js_pos < html_end_pos);
    }

    #[test]
    fn test_inject_script_minimal_html() {
        let html = "<h1>Hello</h1>";
        let result = inject_rss_script(html.to_string());
        assert!(result.contains("rss.js"));
        assert!(result.contains("_reset.css"));
    }

    #[test]
    fn test_inject_script_no_double_inject() {
        let html = "<html><head></head><body></body></html>";
        let result = inject_rss_script(html.to_string());
        assert_eq!(result.matches("rss.js").count(), 1);
        assert_eq!(result.matches("_reset.css").count(), 1);
    }
}
