use crate::proxy::ProxyManager;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::sync::Arc;

pub struct ProxyServer {
    manager: Arc<ProxyManager>,
}

impl ProxyServer {
    pub fn new(manager: Arc<ProxyManager>) -> Self {
        Self { manager }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = self.manager.get_config();
        let addr = ([127, 0, 0, 1], config.port).into();

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
            "Reverse Proxy listening on http://127.0.0.1:{}",
            config.port
        );
        log::info!("Route pattern: {{servername}}.localhost -> 127.0.0.1:{{port}}");

        if let Err(e) = server.await {
            log::error!("Proxy server error: {}", e);
        }

        Ok(())
    }

    // NEU: HTTPS-Server hinzufügen
    pub async fn start_with_https(&self) -> crate::core::prelude::Result<()> {
        let config = self.manager.get_config();
        let https_port = config.port + 443; // 8443

        // Manager EINMAL clonen für beide Tasks
        let manager_for_http = Arc::clone(&self.manager);
        let manager_for_https = Arc::clone(&self.manager);
        let config_clone = config.clone(); // Config für HTTPS Task

        log::info!("Starting HTTP + HTTPS proxy servers...");
        log::info!("  HTTP:  http://127.0.0.1:{}", config.port);
        log::info!("  HTTPS: https://127.0.0.1:{}", https_port);

        // HTTP Server Task
        let http_task = tokio::spawn(async move {
            let proxy_server = ProxyServer::new(manager_for_http);
            if let Err(e) = proxy_server.start().await {
                log::error!("HTTP proxy failed: {}", e);
            }
        });

        // HTTPS Server Task
        let https_task = tokio::spawn(async move {
            // TLS-Setup
            let tls_manager = match crate::server::tls::TlsManager::new(".rss/certs", 365) {
                Ok(manager) => manager,
                Err(e) => {
                    log::error!("TLS manager creation failed: {}", e);
                    return;
                }
            };

            let tls_config = match tls_manager.get_rustls_config("proxy", config_clone.port) {
                Ok(config) => config,
                Err(e) => {
                    log::error!("TLS config failed: {}", e);
                    return;
                }
            };

            // HTTPS Listener
            let listener = match tokio::net::TcpListener::bind(("127.0.0.1", https_port)).await {
                Ok(listener) => listener,
                Err(e) => {
                    log::error!("HTTPS bind failed: {}", e);
                    return;
                }
            };

            let acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
            log::info!("HTTPS proxy listening on https://127.0.0.1:{}", https_port);

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        log::warn!("HTTPS accept failed: {}", e);
                        continue;
                    }
                };

                let acceptor = acceptor.clone();
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

        // Tasks parallel laufen lassen
        tokio::select! {
            _ = http_task => log::error!("HTTP task ended"),
            _ = https_task => log::error!("HTTPS task ended"),
        }

        Ok(())
    }
}

// Rest der handle_proxy_request Funktion bleibt gleich...
pub async fn handle_proxy_request(
    req: Request<Body>,
    manager: Arc<ProxyManager>,
    client: Client<hyper::client::HttpConnector>,
) -> Result<Response<Body>, hyper::Error> {
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "localhost".to_string());

    let subdomain = if let Some(dot_pos) = host.find('.') {
        host[..dot_pos].to_string()
    } else {
        host.clone()
    };

    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();

    if let Some(target_port) = manager.get_target_port(&subdomain).await {
        let target_uri = format!("http://127.0.0.1:{}{}", target_port, path_and_query);

        match target_uri.parse::<Uri>() {
            Ok(uri) => {
                let (mut parts, body) = req.into_parts();
                parts.uri = uri;
                parts.headers.insert(
                    "host",
                    format!("127.0.0.1:{}", target_port).parse().unwrap(),
                );
                let backend_req = Request::from_parts(parts, body);

                match client.request(backend_req).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        log::warn!("Backend request failed for {}.localhost: {}", subdomain, e);
                        Ok(Response::builder()
                            .status(502)
                            .header("content-type", "text/html")
                            .body(Body::from(format!(
                                r#"<!DOCTYPE html>
<html><head><title>Backend Unavailable</title></head>
<body>
<h1>502 Bad Gateway</h1>
<p>Backend server for <strong>{}.localhost</strong> is not responding.</p>
<p>Target: 127.0.0.1:{}</p>
</body></html>"#,
                                subdomain, target_port
                            )))
                            .unwrap())
                    }
                }
            }
            Err(_) => Ok(Response::builder()
                .status(400)
                .body(Body::from("Invalid target URI"))
                .unwrap()),
        }
    } else {
        let routes = manager.get_routes().await;
        let route_list = routes
            .iter()
            .map(|route| {
                format!(
                    r#"<li><a href="http://{}.localhost:{}/">{}.localhost</a> → 127.0.0.1:{}</li>"#,
                    route.subdomain,
                    manager.get_config().port,
                    route.subdomain,
                    route.target_port
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Response::builder()
            .status(404)
            .header("content-type", "text/html")
            .body(Body::from(format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Subdomain Not Found</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 40px; }}
        .container {{ max-width: 600px; margin: 0 auto; }}
        h1 {{ color: #ff4757; }}
        .routes {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin: 20px 0; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ margin: 8px 0; }}
        a {{ color: #0066cc; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        code {{ background: #e9ecef; padding: 2px 6px; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Subdomain '{}.localhost' Not Found</h1>
        <p>The requested subdomain <code>{}.localhost</code> is not configured in the reverse proxy.</p>

        <div class="routes">
            <h3>Available Routes:</h3>
            {}
        </div>

        <p><strong>How to add a new route:</strong></p>
        <pre><code>cargo run server create myapp --port 8080</code></pre>
        <p>This will automatically register <code>myapp.localhost</code> with the proxy.</p>
    </div>
</body>
</html>"#, subdomain, subdomain,
    if routes.is_empty() {
        "<p><em>No routes configured yet. Start a server to see routes here.</em></p>".to_string()
    } else {
        format!("<ul>{}</ul>", route_list)
    })))
            .unwrap())
    }
}
