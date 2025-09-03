// ===== src/server/handlers/web/assets.rs =====
use super::PROXY_PORT;
use crate::server::types::ServerData;
use actix_web::{web, HttpResponse, Result as ActixResult};

pub async fn serve_rss_js(data: web::Data<ServerData>) -> ActixResult<HttpResponse> {
    let js_content = include_str!("../templates/rss/rss.js")
        .replace("{{SERVER_NAME}}", &data.name)
        .replace("{{PORT}}", &data.port.to_string())
        .replace("{{PROXY_PORT}}", &PROXY_PORT.to_string());

    Ok(HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache"))
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
    let reset_css = include_str!("../templates/rss/global-reset.css");

    Ok(HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .insert_header(("Cache-Control", "public, max-age=3600"))
        .body(reset_css))
}
