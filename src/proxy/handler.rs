use crate::proxy::ProxyManager;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::sync::{Arc, OnceLock, RwLock};

// Global TLS acceptor that ACME can hot-reload after provisioning a new certificate
static PROXY_TLS_ACCEPTOR: OnceLock<RwLock<tokio_rustls::TlsAcceptor>> = OnceLock::new();

pub fn reload_proxy_tls(domain: &str) {
    let tls_manager = match crate::server::tls::TlsManager::new(".rss/certs", 365) {
        Ok(m) => m,
        Err(e) => {
            log::error!("TLS reload: manager creation failed: {}", e);
            return;
        }
    };

    // Try LE cert first; fall back to self-signed with production domain SANs.
    // This ensures the proxy always has a cert for the right domain, even if
    // ACME hasn't succeeded yet.
    let config = match tls_manager.get_production_config(domain) {
        Ok(c) => {
            log::info!("TLS reload: loaded Let's Encrypt certificate for {}", domain);
            c
        }
        Err(e) => {
            log::warn!("TLS reload: LE cert not available ({}), trying self-signed for {}", e, domain);
            // Use the proxy's HTTPS port (read from existing config if possible)
            let proxy_port = crate::server::handlers::web::get_proxy_https_port();
            match tls_manager.get_rustls_config_for_domain("proxy", proxy_port, domain) {
                Ok(c) => {
                    log::info!("TLS reload: using self-signed certificate for {}", domain);
                    c
                }
                Err(e2) => {
                    log::error!("TLS reload: all cert loading failed: {}", e2);
                    return;
                }
            }
        }
    };

    let new_acceptor = tokio_rustls::TlsAcceptor::from(config);
    match PROXY_TLS_ACCEPTOR.get() {
        Some(lock) => {
            if let Ok(mut guard) = lock.write() {
                *guard = new_acceptor;
                log::info!("TLS: Proxy certificate hot-reloaded for {}", domain);
            }
        }
        None => {
            let _ = PROXY_TLS_ACCEPTOR.set(RwLock::new(new_acceptor));
        }
    }
}

pub struct ProxyServer {
    manager: Arc<ProxyManager>,
}

impl ProxyServer {
    pub fn new(manager: Arc<ProxyManager>) -> Self {
        Self { manager }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = self.manager.get_config();
        let addr: std::net::SocketAddr = format!("{}:{}", config.bind_address, config.port)
            .parse()
            .map_err(|e| format!("Invalid bind address: {}", e))?;

        let manager = Arc::clone(&self.manager);

        let make_svc = make_service_fn(move |_conn| {
            let manager = Arc::clone(&manager);
            let client = Client::new();

            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let manager = Arc::clone(&manager);
                    let client = client.clone();
                    handle_proxy_request(req, manager, client)
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        log::info!(
            "Reverse Proxy listening on http://{}:{}",
            config.bind_address,
            config.port
        );
        log::info!(
            "Route pattern: {{servername}}.{{domain}} -> {}:{{port}}",
            config.bind_address
        );

        if let Err(e) = server.await {
            log::error!("Proxy server error: {}", e);
        }

        Ok(())
    }

    pub async fn start_with_https(&self) -> crate::core::prelude::Result<()> {
        let config = self.manager.get_config();
        let https_port = config.port + config.https_port_offset;

        let manager_for_http = Arc::clone(&self.manager);
        let manager_for_https = Arc::clone(&self.manager);
        let config_clone = config.clone();
        // Use values from ProxyConfig directly — no need to re-load config
        let production_domain = config.production_domain.clone();
        let use_lets_encrypt = config.use_lets_encrypt;

        log::info!("Starting HTTP + HTTPS proxy servers...");
        log::info!("  HTTP:  http://{}:{}", config.bind_address, config.port);
        log::info!("  HTTPS: https://{}:{}", config.bind_address, https_port);
        if use_lets_encrypt {
            log::info!("  TLS:   Let's Encrypt for *.{}", production_domain);
        }

        let http_task = tokio::spawn(async move {
            let proxy_server = ProxyServer::new(manager_for_http);
            if let Err(e) = proxy_server.start().await {
                log::error!("HTTP proxy failed: {}", e);
            }
        });

        let https_task =
            tokio::spawn(async move {
                let tls_manager = match crate::server::tls::TlsManager::new(".rss/certs", 365) {
                    Ok(manager) => manager,
                    Err(e) => {
                        log::error!("TLS manager creation failed: {}", e);
                        return;
                    }
                };

                // If production domain is set, remove any stale self-signed proxy certs
                // that may have been generated for a different domain (e.g. *.localhost).
                // This ensures the fallback cert always matches the current domain.
                if production_domain != "localhost" {
                    let _ = tls_manager.remove_certificate("proxy", config_clone.port);
                    // Also remove stale proxy-443 cert from old buggy fallback code
                    let _ = tls_manager.remove_certificate("proxy", 443);
                }

                // Use Let's Encrypt certificate if available, otherwise self-signed
                let tls_config = if use_lets_encrypt && production_domain != "localhost" {
                    match tls_manager.get_production_config(&production_domain) {
                        Ok(config) => {
                            log::info!("TLS: Using Let's Encrypt certificate for {}", production_domain);
                            config
                        }
                        Err(e) => {
                            log::warn!("TLS: Let's Encrypt cert not ready ({}), using self-signed for {}", e, production_domain);
                            match tls_manager.get_rustls_config_for_domain(
                                "proxy", config_clone.port, &production_domain,
                            ) {
                                Ok(c) => c,
                                Err(e) => { log::error!("TLS config failed: {}", e); return; }
                            }
                        }
                    }
                } else {
                    match tls_manager.get_rustls_config_for_domain(
                        "proxy", config_clone.port, &production_domain,
                    ) {
                        Ok(config) => config,
                        Err(e) => { log::error!("TLS config failed: {}", e); return; }
                    }
                };

                let listener =
                    match tokio::net::TcpListener::bind((&*config_clone.bind_address, https_port))
                        .await
                    {
                        Ok(listener) => listener,
                        Err(e) => {
                            log::error!("HTTPS bind failed: {}", e);
                            return;
                        }
                    };

                let initial_acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
                // Store in global so ACME can hot-reload it later
                match PROXY_TLS_ACCEPTOR.get() {
                    Some(lock) => { if let Ok(mut g) = lock.write() { *g = initial_acceptor.clone(); } }
                    None => { let _ = PROXY_TLS_ACCEPTOR.set(RwLock::new(initial_acceptor.clone())); }
                }
                log::info!(
                    "HTTPS proxy listening on https://{}:{}",
                    config_clone.bind_address,
                    https_port
                );

                loop {
                    let (stream, _) = match listener.accept().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            log::warn!("HTTPS accept failed: {}", e);
                            continue;
                        }
                    };

                    // Read current acceptor (may have been hot-reloaded by ACME)
                    let acceptor = PROXY_TLS_ACCEPTOR
                        .get()
                        .and_then(|lock| lock.read().ok())
                        .map(|a| a.clone())
                        .unwrap_or_else(|| initial_acceptor.clone());
                    let manager = Arc::clone(&manager_for_https);

                    tokio::spawn(async move {
                        let tls_stream = match acceptor.accept(stream).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                log::debug!("TLS handshake failed: {}", e);
                                return;
                            }
                        };

                        let service = hyper::service::service_fn(move |req| {
                            let manager = Arc::clone(&manager);
                            let client = hyper::Client::new();
                            handle_proxy_request(req, manager, client)
                        });

                        if let Err(e) = hyper::server::conn::Http::new()
                            .serve_connection(tls_stream, service)
                            .await
                        {
                            log::debug!("HTTPS connection error: {}", e);
                        }
                    });
                }
            });

        // Run both tasks concurrently
        tokio::select! {
            _ = http_task => log::error!("HTTP task ended"),
            _ = https_task => log::error!("HTTPS task ended"),
        }

        Ok(())
    }
}

use crate::core::helpers::html_escape;

pub async fn handle_proxy_request(
    req: Request<Body>,
    manager: Arc<ProxyManager>,
    client: Client<hyper::client::HttpConnector>,
) -> Result<Response<Body>, hyper::Error> {
    let config = manager.get_config();
    let domain = config.production_domain.clone();

    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| domain.clone());

    // Extract port from host header — use the external port the client sees, not the internal config port
    let (host_no_port, external_port_suffix) = if let Some(colon) = host.rfind(':') {
        let port_str = &host[colon + 1..];
        if port_str.parse::<u16>().is_ok() {
            (host[..colon].to_string(), format!(":{}", port_str))
        } else {
            (host.clone(), String::new())
        }
    } else {
        (host.clone(), String::new())
    };

    // Extract subdomain by properly matching against the production domain
    let subdomain = if host_no_port == domain
        || host_no_port == format!("www.{}", domain)
        || host_no_port == "localhost"
    {
        // Bare domain, www, or localhost — no subdomain
        String::new()
    } else if let Some(stripped) = host_no_port.strip_suffix(&format!(".{}", domain)) {
        stripped.to_string()
    } else if let Some(stripped) = host_no_port.strip_suffix(".localhost") {
        stripped.to_string()
    } else {
        // Fallback for IP or unknown host — use first segment
        if let Some(dot_pos) = host_no_port.find('.') {
            host_no_port[..dot_pos].to_string()
        } else {
            host_no_port.clone()
        }
    };

    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();

    log::info!(
        "Proxy Request: Host='{}' -> Subdomain='{}' Path='{}'",
        host,
        if subdomain.is_empty() { "(bare domain)" } else { &subdomain },
        path_and_query
    );

    // Analytics tracking
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let proxy_user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    crate::server::analytics::track_request(
        &subdomain,
        &path_and_query,
        &client_ip,
        &proxy_user_agent,
    );

    // ACME challenges must be served BEFORE any redirects (Let's Encrypt validates on bare domain)
    if path_and_query.starts_with("/.well-known/acme-challenge/") {
        let token = path_and_query
            .strip_prefix("/.well-known/acme-challenge/")
            .unwrap_or("");
        if let Some(key_auth) = crate::server::acme::get_challenge_response(token) {
            log::info!("ACME: Serving challenge response for token {}", token);
            return Ok(Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body(Body::from(key_auth))
                .unwrap_or_else(|_| Response::new(Body::empty())));
        }
    }

    // Handle bare domain — redirect to default subdomain
    if subdomain.is_empty() {
        if manager.get_target_port("default").await.is_some() {
            return Ok(Response::builder()
                .status(302)
                .header(
                    "location",
                    format!("http://default.{}{}/", domain, external_port_suffix),
                )
                .body(Body::empty())
                .expect("redirect response"));
        }
        // No default server — show welcome page
        let routes = manager.get_routes().await;
        let route_links = routes
            .iter()
            .map(|r| {
                format!(
                    r#"<a href="http://{sub}.{dom}{port}/" style="display:inline-block;padding:10px 20px;background:rgba(108,99,255,0.15);border:1px solid rgba(108,99,255,0.3);border-radius:8px;color:#6c63ff;text-decoration:none;font-weight:500;margin:4px;">{sub}.{dom}</a>"#,
                    sub = r.subdomain,
                    dom = domain,
                    port = external_port_suffix
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        return Ok(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(Body::from(format!(
                r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Rush Sync Server</title>
<style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#0a0a0f;color:#e4e4ef;display:flex;align-items:center;justify-content:center;min-height:100vh;padding:20px}}.c{{text-align:center;max-width:600px}}h1{{font-size:clamp(32px,5vw,48px);font-weight:800;letter-spacing:-1px;margin-bottom:12px}}h1 span{{color:#6c63ff}}.sub{{color:#8888a0;font-size:16px;margin-bottom:32px}}.routes{{margin:24px 0;display:flex;flex-wrap:wrap;justify-content:center;gap:8px}}.info{{color:#8888a0;font-size:14px;margin-top:24px}}a.gh{{color:#6c63ff;text-decoration:none}}</style>
</head><body><div class="c">
<h1>RUSH<span>.</span>SYNC<span>.</span>SERVER</h1>
<p class="sub">{}</p>
<div class="routes">{}</div>
<p class="info">Powered by Rush Sync Server v0.3.8</p>
<p class="info" style="margin-top:8px"><a class="gh" href="https://github.com/LEVOGNE/rush.sync.server">GitHub</a></p>
</div></body></html>"#,
                if routes.is_empty() {
                    "No servers are running yet. Create one to get started.".to_string()
                } else {
                    format!("{} active server{} on this domain:", routes.len(), if routes.len() == 1 { "" } else { "s" })
                },
                if routes.is_empty() {
                    String::new()
                } else {
                    route_links
                }
            )))
            .expect("welcome response"));
    }

    // Special subdomains served directly by the proxy
    if subdomain == "blog" {
        let blog = include_str!("blog.html")
            .replace("{{DOMAIN}}", &html_escape(&domain))
            .replace("{{PORT_SUFFIX}}", &external_port_suffix);
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(blog))
            .expect("blog response"));
    }

    // Check if route exists
    let routes = manager.get_routes().await;
    log::info!(
        "Available routes: {:?}",
        routes.iter().map(|r| &r.subdomain).collect::<Vec<_>>()
    );

    if let Some(target_port) = manager.get_target_port(&subdomain).await {
        let target_uri = format!("http://127.0.0.1:{}{}", target_port, path_and_query);

        match target_uri.parse::<Uri>() {
            Ok(uri) => {
                let (mut parts, body) = req.into_parts();
                parts.uri = uri;
                parts.headers.insert(
                    "host",
                    format!("127.0.0.1:{}", target_port)
                        .parse()
                        .unwrap_or_else(|_| hyper::header::HeaderValue::from_static("localhost")),
                );
                let backend_req = Request::from_parts(parts, body);

                match client.request(backend_req).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        log::warn!("Backend request failed for {}.{}: {}", subdomain, domain, e);
                        Ok(Response::builder()
                            .status(502)
                            .header("content-type", "text/html")
                            .body(Body::from(format!(
                                r#"<!DOCTYPE html>
<html><head><title>Backend Unavailable</title></head>
<body>
<h1>502 Bad Gateway</h1>
<p>Backend server for <strong>{}.{}</strong> is not responding.</p>
<p>Target: 127.0.0.1:{}</p>
</body></html>"#,
                                html_escape(&subdomain),
                                html_escape(&domain),
                                target_port
                            )))
                            .expect("static 502 response"))
                    }
                }
            }
            Err(_) => Ok(Response::builder()
                .status(400)
                .body(Body::from("Invalid target URI"))
                .expect("static 400 response")),
        }
    } else {
        let routes = manager.get_routes().await;
        let routes_html = if routes.is_empty() {
            r#"<div class="no-routes">No servers are running on this domain yet.</div>"#.to_string()
        } else {
            format!(
                r#"<p class="lbl">Active Servers on this Domain</p><div class="route-grid">{}</div>"#,
                routes.iter().map(|r| format!(
                    r#"<a href="http://{sub}.{dom}{port}/">{sub}.{dom} <span class="arrow">&rarr;</span></a>"#,
                    sub = r.subdomain, dom = domain, port = external_port_suffix
                )).collect::<Vec<_>>().join("\n")
            )
        };

        let showroom = include_str!("showroom.html")
            .replace("{{SUBDOMAIN}}", &html_escape(&subdomain))
            .replace("{{DOMAIN}}", &html_escape(&domain))
            .replace("{{PORT_SUFFIX}}", &external_port_suffix)
            .replace("{{ROUTES_HTML}}", &routes_html);

        Ok(Response::builder()
            .status(200)
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(showroom))
            .expect("showroom response"))
    }
}
