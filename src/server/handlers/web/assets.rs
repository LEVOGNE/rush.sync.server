use super::ServerDataWithConfig;
use actix_web::{web, HttpResponse, Result as ActixResult};

/// Escape a string for safe embedding inside JavaScript string literals.
fn js_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('<', "\\x3c")
        .replace('>', "\\x3e")
        .replace('&', "\\x26")
}

pub async fn serve_rss_js(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("../templates/rss/rss.js")
        .replace("{{SERVER_NAME}}", &js_escape(&data.server.name))
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string());

    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(js_content))
}

// App Controller Module
pub async fn serve_rush_app_js(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("../templates/rss/js/rush-app.js")
        .replace("{{SERVER_NAME}}", &js_escape(&data.server.name))
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string());
    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(js_content))
}

pub async fn serve_rush_api_js(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("../templates/rss/js/rush-api.js")
        .replace("{{SERVER_NAME}}", &js_escape(&data.server.name))
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string());

    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(js_content))
}

pub async fn serve_rush_ui_js(data: web::Data<ServerDataWithConfig>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("../templates/rss/js/rush-ui.js")
        .replace("{{SERVER_NAME}}", &js_escape(&data.server.name))
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string());

    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(js_content))
}

pub async fn serve_system_css() -> ActixResult<HttpResponse> {
    let css_content = include_str!("../templates/rss/style.css");

    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
        .body(css_content))
}

pub async fn serve_system_favicon() -> ActixResult<HttpResponse> {
    let favicon_content = include_str!("../templates/rss/favicon.svg");

    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(favicon_content))
}

pub async fn serve_quicksand_font(req: actix_web::HttpRequest) -> ActixResult<HttpResponse> {
    let path = req
        .match_info()
        .get("font")
        .unwrap_or("Kenyan_Coffee_Bd.otf");

    let valid_fonts = [
        "Kenyan_Coffee_Bd_It.otf",
        "Kenyan_Coffee_Bd.otf",
        "Kenyan_Coffee_Rg_It.otf",
        "Kenyan_Coffee_Rg.otf",
    ];

    if !valid_fonts.contains(&path) {
        return Ok(HttpResponse::NotFound().body("Font not found"));
    }

    let font_data: &[u8] = match path {
        "Kenyan_Coffee_Bd_It.otf" => {
            include_bytes!("../templates/rss/fonts/Kenyan_Coffee_Bd_It.otf").as_slice()
        }
        "Kenyan_Coffee_Bd.otf" => {
            include_bytes!("../templates/rss/fonts/Kenyan_Coffee_Bd.otf").as_slice()
        }
        "Kenyan_Coffee_Rg_It.otf" => {
            include_bytes!("../templates/rss/fonts/Kenyan_Coffee_Rg_It.otf").as_slice()
        }
        "Kenyan_Coffee_Rg.otf" => {
            include_bytes!("../templates/rss/fonts/Kenyan_Coffee_Rg.otf").as_slice()
        }
        _ => return Ok(HttpResponse::NotFound().body("Font not found")),
    };

    Ok(HttpResponse::Ok()
        .content_type("font/otf")
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .body(font_data))
}

pub async fn serve_global_reset_css() -> ActixResult<HttpResponse> {
    let reset_css = include_str!("../templates/rss/_reset.css");

    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=3600"))
        .body(reset_css))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_escape_basic() {
        assert_eq!(js_escape("hello"), "hello");
    }

    #[test]
    fn test_js_escape_quotes() {
        assert_eq!(js_escape(r#"it's a "test""#), r#"it\'s a \"test\""#);
    }

    #[test]
    fn test_js_escape_backslash() {
        assert_eq!(js_escape(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_js_escape_html_tags() {
        assert_eq!(js_escape("<script>"), "\\x3cscript\\x3e");
        assert_eq!(js_escape("</script>"), "\\x3c/script\\x3e");
    }

    #[test]
    fn test_js_escape_ampersand() {
        assert_eq!(js_escape("a&b"), "a\\x26b");
    }

    #[test]
    fn test_js_escape_xss_payload() {
        let payload = r#"</script><script>alert('xss')</script>"#;
        let escaped = js_escape(payload);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(escaped.contains("\\x3c"));
        assert!(escaped.contains("\\x3e"));
    }

    #[test]
    fn test_js_escape_empty() {
        assert_eq!(js_escape(""), "");
    }

    #[test]
    fn test_js_escape_server_name() {
        assert_eq!(js_escape("my-server-01"), "my-server-01");
        assert_eq!(js_escape("test<>server"), "test\\x3c\\x3eserver");
    }
}
