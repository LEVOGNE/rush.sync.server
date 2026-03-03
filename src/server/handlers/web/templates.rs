use super::ServerDataWithConfig;
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};

pub async fn serve_system_dashboard(
    req: HttpRequest,
    data: web::Data<ServerDataWithConfig>,
) -> ActixResult<HttpResponse> {
    // Check PIN protection
    let server_dir =
        crate::server::settings::ServerSettings::get_server_dir(&data.server.name, data.server.port);
    if let Some(ref dir) = server_dir {
        let settings = crate::server::settings::ServerSettings::load(dir);
        if settings.pin_enabled && !settings.pin_code.is_empty() {
            let expected_token = format!("rss-pin-{}-{}", data.server.name, data.server.port);
            let has_valid_cookie = req
                .cookie("rss_pin")
                .map(|c| c.value() == expected_token)
                .unwrap_or(false);

            if !has_valid_cookie {
                return serve_pin_page(&data);
            }
        }
    }

    let template = include_str!("../templates/rss/dashboard.html");

    let html_content = template
        .replace("{{SERVER_NAME}}", &data.server.name)
        .replace("{{PORT}}", &data.server.port.to_string())
        .replace("{{PROXY_PORT}}", &data.proxy_http_port.to_string())
        .replace("{{PROXY_HTTPS_PORT}}", &data.proxy_https_port.to_string())
        .replace("{{VERSION}}", crate::server::config::get_server_version())
        .replace("{{CREATION_TIME}}", &chrono::Local::now().to_rfc3339());

    let html_with_script = crate::server::handlers::web::server::inject_rss_script(html_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_with_script))
}

fn serve_pin_page(_data: &ServerDataWithConfig) -> ActixResult<HttpResponse> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Locked</title>
<link rel="icon" href="/.rss/favicon.svg" type="image/svg+xml">
<style>
*{margin:0;padding:0;box-sizing:border-box;}
body{background:#000;min-height:100vh;display:flex;align-items:center;justify-content:center;}
form{width:200px;}
input{
  width:100%;
  background:transparent;
  border:none;
  border-bottom:1px solid #222;
  color:#fff;
  font-family:monospace;
  font-size:16px;
  text-align:center;
  letter-spacing:6px;
  padding:12px 0;
  outline:none;
  -webkit-text-security:disc;
  transition:border-color 0.2s;
}
input:focus{border-bottom-color:#444;}
input.err{border-bottom-color:#611;}
</style>
</head>
<body>
<form id="f">
  <input type="password" id="i" autocomplete="off" autofocus spellcheck="false">
</form>
<script>
document.getElementById('f').addEventListener('submit',function(e){
  e.preventDefault();
  var i=document.getElementById('i');
  fetch('/api/pin/verify',{
    method:'POST',
    headers:{'Content-Type':'application/json'},
    body:JSON.stringify({pin:i.value})
  }).then(function(r){
    if(r.ok){location.reload();}
    else{i.classList.add('err');i.value='';setTimeout(function(){i.classList.remove('err');},800);}
  }).catch(function(){i.classList.add('err');});
});
</script>
</body>
</html>"#;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
