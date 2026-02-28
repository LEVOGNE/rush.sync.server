use rush_sync_server::{create_default_registry, CommandHandler};

#[test]
fn test_core_functionality() {
    let handler = CommandHandler::new();

    assert!(!handler.list_commands().is_empty());

    let test_cases = [
        ("version", true),
        ("clear", true),
        ("exit", true),
        ("unknown_xyz", false),
    ];

    for (command, should_succeed) in test_cases {
        let result = handler.handle_input(command);
        assert_eq!(
            result.success, should_succeed,
            "Command '{}' failed",
            command
        );
    }
}

#[tokio::test]
async fn test_async_commands() {
    let handler = CommandHandler::new();
    let result = handler.handle_input_async("lang").await;
    assert!(result.success);
}

#[test]
fn test_registry() {
    let registry = create_default_registry();
    assert!(!registry.is_empty());
    assert!(registry.find_command("version").is_some());
}

// Command-Trait safety: all registered commands must have name, description, and matches
#[test]
fn test_all_commands_have_metadata() {
    let registry = create_default_registry();
    let commands = registry.list_commands();

    for (name, description) in &commands {
        assert!(!name.is_empty(), "Command has empty name");
        assert!(
            !description.is_empty(),
            "Command '{}' has empty description",
            name
        );
    }
}

// Parsing tests for BulkMode
#[test]
fn test_bulk_parsing_single() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    match parse_bulk_args(&["myserver"]) {
        BulkMode::Single(id) => assert_eq!(id, "myserver"),
        other => panic!("Expected Single, got {:?}", other),
    }
}

#[test]
fn test_bulk_parsing_all() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    match parse_bulk_args(&["all"]) {
        BulkMode::All => {}
        other => panic!("Expected All, got {:?}", other),
    }

    match parse_bulk_args(&["ALL"]) {
        BulkMode::All => {}
        other => panic!("Expected All for uppercase, got {:?}", other),
    }
}

#[test]
fn test_bulk_parsing_range() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    match parse_bulk_args(&["1-3"]) {
        BulkMode::Range(start, end) => {
            assert_eq!(start, 1);
            assert_eq!(end, 3);
        }
        other => panic!("Expected Range, got {:?}", other),
    }
}

#[test]
fn test_bulk_parsing_invalid_range() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    // Reversed range
    match parse_bulk_args(&["5-2"]) {
        BulkMode::Invalid(_) => {}
        other => panic!("Expected Invalid for reversed range, got {:?}", other),
    }

    // Zero in range
    match parse_bulk_args(&["0-3"]) {
        BulkMode::Invalid(_) => {}
        other => panic!("Expected Invalid for zero start, got {:?}", other),
    }

    // Too many arguments
    match parse_bulk_args(&["1", "2"]) {
        BulkMode::Invalid(_) => {}
        other => panic!("Expected Invalid for too many args, got {:?}", other),
    }
}

#[test]
fn test_bulk_parsing_range_too_large() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    match parse_bulk_args(&["1-600"]) {
        BulkMode::Invalid(msg) => assert!(msg.contains("Maximum 500")),
        other => panic!("Expected Invalid for large range, got {:?}", other),
    }
}

#[test]
fn test_bulk_parsing_name_with_dash() {
    use rush_sync_server::commands::parsing::{parse_bulk_args, BulkMode};

    // "my-server" should be Single, not a range
    match parse_bulk_args(&["my-server"]) {
        BulkMode::Single(id) => assert_eq!(id, "my-server"),
        other => panic!("Expected Single for name with dash, got {:?}", other),
    }
}

// Config defaults test
#[tokio::test]
async fn test_config_default_values() {
    let config = rush_sync_server::Config::default();
    assert_eq!(config.server.port_range_start, 8080);
    assert_eq!(config.server.port_range_end, 8999);
    assert_eq!(config.server.max_concurrent, 50);
    assert!(config.server.workers >= 1);
    assert!(config.server.shutdown_timeout > 0);
}

// Security: path traversal detection
#[test]
fn test_server_name_validation() {
    use rush_sync_server::server::utils::validation::validate_server_name;

    assert!(validate_server_name("myserver").is_ok());
    assert!(validate_server_name("my-server-001").is_ok());
    assert!(validate_server_name("rss-001").is_ok());

    // Invalid names
    assert!(validate_server_name("").is_err());
    assert!(validate_server_name("a".repeat(65).as_str()).is_err());
}

// i18n basic tests
#[test]
fn test_translation_missing_key() {
    let result = rush_sync_server::i18n::get_translation("nonexistent.key.xyz", &[]);
    assert!(result.starts_with("Missing:"));
}

#[test]
fn test_available_languages() {
    let languages = rush_sync_server::i18n::get_available_languages();
    assert!(!languages.is_empty());
}

// Handler edge cases
#[test]
fn test_empty_input() {
    let handler = CommandHandler::new();
    let result = handler.handle_input("");
    assert!(!result.success);
    assert!(result.message.is_empty());
}

#[test]
fn test_whitespace_input() {
    let handler = CommandHandler::new();
    let result = handler.handle_input("   ");
    assert!(!result.success);
    assert!(result.message.is_empty());
}

#[test]
fn test_long_input_rejected() {
    let handler = CommandHandler::new();
    let long_input = "a".repeat(1001);
    let result = handler.handle_input(&long_input);
    assert!(!result.success);
    assert!(result.message.contains("too long"));
}

// =============================================================================
// Web Handler Tests (actix-web)
// =============================================================================

mod web_handler_tests {
    use actix_web::{test, web, App};
    use rush_sync_server::server::handlers::web::{
        close_browser_handler, health_handler, info_handler, message_handler, messages_handler,
        ping_handler, serve_global_reset_css, serve_quicksand_font, serve_rss_js, serve_system_css,
        serve_system_favicon, status_handler, ServerDataWithConfig,
    };
    use rush_sync_server::server::types::ServerData;

    fn test_server_data() -> web::Data<ServerDataWithConfig> {
        web::Data::new(ServerDataWithConfig {
            server: ServerData {
                id: "test-server-id".to_string(),
                port: 8080,
                name: "testserver".to_string(),
            },
            proxy_http_port: 3000,
            proxy_https_port: 3443,
        })
    }

    // --- Health Handler ---

    #[actix_web::test]
    async fn test_health_handler_returns_200() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/health", web::get().to(health_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_health_handler_json_format() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/health", web::get().to(health_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/health").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "healthy");
        assert!(resp["timestamp"].is_number());
        assert_eq!(resp["hot_reload"], "active");
        assert_eq!(resp["static_files"], "enabled");
    }

    // --- Ping Handler ---

    #[actix_web::test]
    async fn test_ping_handler_returns_pong() {
        let app =
            test::init_service(App::new().route("/api/ping", web::post().to(ping_handler))).await;

        let req = test::TestRequest::post().uri("/api/ping").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "pong");
        assert_eq!(resp["server"], "rush-sync-server");
        assert!(resp["timestamp"].is_number());
    }

    // --- Status Handler ---

    #[actix_web::test]
    async fn test_status_handler_returns_server_info() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/status", web::get().to(status_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/status").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "running");
        assert_eq!(resp["server_id"], "test-server-id");
        assert_eq!(resp["server_name"], "testserver");
        assert_eq!(resp["port"], 8080);
        assert_eq!(resp["proxy_port"], 3443);
        assert!(resp["uptime_seconds"].is_number());
        assert_eq!(resp["hot_reload"], true);
    }

    #[actix_web::test]
    async fn test_status_handler_urls() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/status", web::get().to(status_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/status").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["urls"]["http"], "http://127.0.0.1:8080");
        assert_eq!(resp["urls"]["proxy"], "https://testserver.localhost:3443");
    }

    // --- Info Handler ---

    #[actix_web::test]
    async fn test_info_handler_returns_endpoints() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/info", web::get().to(info_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/info").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["name"], "Rush Sync Server");
        assert_eq!(resp["server_name"], "testserver");
        assert_eq!(resp["port"], 8080);

        let endpoints = resp["endpoints"].as_array().unwrap();
        assert_eq!(endpoints.len(), 10);
    }

    #[actix_web::test]
    async fn test_info_handler_certificate_paths() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/api/info", web::get().to(info_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/info").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            resp["certificate"]["cert_file"],
            ".rss/certs/testserver-8080.cert"
        );
        assert_eq!(
            resp["certificate"]["key_file"],
            ".rss/certs/testserver-8080.key"
        );
        assert_eq!(resp["certificate"]["common_name"], "testserver.localhost");
    }

    // --- Message Handler ---

    #[actix_web::test]
    async fn test_message_handler_stores_message() {
        let app =
            test::init_service(App::new().route("/api/message", web::post().to(message_handler)))
                .await;

        let req = test::TestRequest::post()
            .uri("/api/message")
            .set_json(serde_json::json!({
                "message": "Hello World",
                "from": "test-client",
                "timestamp": "2026-01-01T00:00:00Z"
            }))
            .to_request();

        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "received");
        assert!(resp["message_id"].is_number());
        assert!(resp["timestamp"].is_number());
    }

    #[actix_web::test]
    async fn test_message_handler_defaults() {
        let app =
            test::init_service(App::new().route("/api/message", web::post().to(message_handler)))
                .await;

        let req = test::TestRequest::post()
            .uri("/api/message")
            .set_json(serde_json::json!({}))
            .to_request();

        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["status"], "received");
    }

    // --- Messages Handler ---

    #[actix_web::test]
    async fn test_messages_handler_returns_list() {
        let app =
            test::init_service(App::new().route("/api/messages", web::get().to(messages_handler)))
                .await;

        let req = test::TestRequest::get().uri("/api/messages").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "success");
        assert!(resp["messages"].is_array());
        assert!(resp["count"].is_number());
    }

    // --- Close Browser Handler ---

    #[actix_web::test]
    async fn test_close_browser_handler_returns_html() {
        let app = test::init_service(
            App::new().route("/api/close-browser", web::get().to(close_browser_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/close-browser")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);

        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/html"));

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("window.close()"));
        assert!(body_str.contains("<script>"));
    }

    // --- CSS Asset ---

    #[actix_web::test]
    async fn test_serve_system_css() {
        let app = test::init_service(
            App::new().route("/.rss/style.css", web::get().to(serve_system_css)),
        )
        .await;

        let req = test::TestRequest::get().uri("/.rss/style.css").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/css"));

        let cache = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cache, "no-cache");
    }

    // --- Favicon ---

    #[actix_web::test]
    async fn test_serve_favicon() {
        let app = test::init_service(
            App::new().route("/.rss/favicon.svg", web::get().to(serve_system_favicon)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.rss/favicon.svg")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "image/svg+xml");
    }

    // --- Font Serving ---

    #[actix_web::test]
    async fn test_serve_valid_font() {
        let app = test::init_service(
            App::new().route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.rss/fonts/Kenyan_Coffee_Bd.otf")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "font/otf");

        let cache = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(cache.contains("immutable"));

        let cors = resp
            .headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cors, "*");
    }

    #[actix_web::test]
    async fn test_serve_invalid_font_returns_404() {
        let app = test::init_service(
            App::new().route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.rss/fonts/malicious.otf")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_serve_all_valid_fonts() {
        let valid_fonts = [
            "Kenyan_Coffee_Bd.otf",
            "Kenyan_Coffee_Bd_It.otf",
            "Kenyan_Coffee_Rg.otf",
            "Kenyan_Coffee_Rg_It.otf",
        ];

        let app = test::init_service(
            App::new().route("/.rss/fonts/{font}", web::get().to(serve_quicksand_font)),
        )
        .await;

        for font in valid_fonts {
            let req = test::TestRequest::get()
                .uri(&format!("/.rss/fonts/{}", font))
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200, "Font {} should be served", font);

            let body = test::read_body(resp).await;
            assert!(!body.is_empty(), "Font {} should have content", font);
        }
    }

    // --- Reset CSS ---

    #[actix_web::test]
    async fn test_serve_reset_css() {
        let app = test::init_service(
            App::new().route("/.rss/_reset.css", web::get().to(serve_global_reset_css)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/.rss/_reset.css")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let cache = resp
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(cache.contains("public"));
        assert!(cache.contains("max-age=3600"));
    }

    // --- JavaScript with Template Variables ---

    #[actix_web::test]
    async fn test_serve_rss_js_template_replacement() {
        let data = test_server_data();
        let app = test::init_service(
            App::new()
                .app_data(data)
                .route("/rss.js", web::get().to(serve_rss_js)),
        )
        .await;

        let req = test::TestRequest::get().uri("/rss.js").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);

        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("application/javascript"));

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Template variables should be replaced
        assert!(!body_str.contains("{{SERVER_NAME}}"));
        assert!(!body_str.contains("{{PORT}}"));
        assert!(!body_str.contains("{{PROXY_PORT}}"));
        assert!(!body_str.contains("{{PROXY_HTTPS_PORT}}"));

        // Actual values should be present
        assert!(body_str.contains("testserver"));
        assert!(body_str.contains("8080"));
    }

    // --- XSS Prevention in JS Templates ---

    #[actix_web::test]
    async fn test_js_template_xss_prevention() {
        let malicious_data = web::Data::new(ServerDataWithConfig {
            server: ServerData {
                id: "test-id".to_string(),
                port: 8080,
                name: "<script>alert('xss')</script>".to_string(),
            },
            proxy_http_port: 3000,
            proxy_https_port: 3443,
        });

        let app = test::init_service(
            App::new()
                .app_data(malicious_data)
                .route("/rss.js", web::get().to(serve_rss_js)),
        )
        .await;

        let req = test::TestRequest::get().uri("/rss.js").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // The raw <script> tag must not appear in the output
        assert!(!body_str.contains("<script>alert"));
        // It should be escaped
        assert!(body_str.contains("\\x3cscript\\x3e"));
    }
}

// =============================================================================
// Proxy Manager Tests
// =============================================================================

mod proxy_tests {
    use rush_sync_server::proxy::{ProxyConfig, ProxyManager};

    fn test_proxy_config() -> ProxyConfig {
        ProxyConfig {
            enabled: true,
            port: 3000,
            https_port_offset: 443,
            bind_address: "127.0.0.1".to_string(),
            health_check_interval: 30,
            timeout_ms: 5000,
        }
    }

    #[tokio::test]
    async fn test_proxy_manager_add_route() {
        let manager = ProxyManager::new(test_proxy_config());
        manager.add_route("myapp", "server-1", 8080).await.unwrap();

        let port = manager.get_target_port("myapp").await;
        assert_eq!(port, Some(8080));
    }

    #[tokio::test]
    async fn test_proxy_manager_remove_route() {
        let manager = ProxyManager::new(test_proxy_config());
        manager.add_route("myapp", "server-1", 8080).await.unwrap();
        manager.remove_route("myapp").await.unwrap();

        let port = manager.get_target_port("myapp").await;
        assert_eq!(port, None);
    }

    #[tokio::test]
    async fn test_proxy_manager_multiple_routes() {
        let manager = ProxyManager::new(test_proxy_config());
        manager.add_route("app1", "server-1", 8080).await.unwrap();
        manager.add_route("app2", "server-2", 8081).await.unwrap();
        manager.add_route("app3", "server-3", 8082).await.unwrap();

        assert_eq!(manager.get_target_port("app1").await, Some(8080));
        assert_eq!(manager.get_target_port("app2").await, Some(8081));
        assert_eq!(manager.get_target_port("app3").await, Some(8082));

        let routes = manager.get_routes().await;
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn test_proxy_manager_unknown_subdomain() {
        let manager = ProxyManager::new(test_proxy_config());
        let port = manager.get_target_port("nonexistent").await;
        assert_eq!(port, None);
    }

    #[tokio::test]
    async fn test_proxy_manager_route_overwrite() {
        let manager = ProxyManager::new(test_proxy_config());
        manager.add_route("myapp", "server-1", 8080).await.unwrap();
        manager.add_route("myapp", "server-2", 9090).await.unwrap();

        // Should have the latest port
        assert_eq!(manager.get_target_port("myapp").await, Some(9090));
    }

    #[tokio::test]
    async fn test_proxy_manager_get_routes_empty() {
        let manager = ProxyManager::new(test_proxy_config());
        let routes = manager.get_routes().await;
        assert!(routes.is_empty());
    }

    #[tokio::test]
    async fn test_proxy_config_defaults() {
        let config = ProxyConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 3000);
        assert_eq!(config.https_port_offset, 443);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_proxy_config_https_port_calculation() {
        let config = test_proxy_config();
        let https_port = config.port + config.https_port_offset;
        assert_eq!(https_port, 3443);
    }
}

// =============================================================================
// Server Types Tests
// =============================================================================

mod server_type_tests {
    use rush_sync_server::server::types::{ServerContext, ServerInfo, ServerStatus};

    #[test]
    fn test_server_status_display() {
        assert_eq!(format!("{}", ServerStatus::Running), "RUNNING");
        assert_eq!(format!("{}", ServerStatus::Stopped), "STOPPED");
        assert_eq!(format!("{}", ServerStatus::Failed), "FAILED");
    }

    #[test]
    fn test_server_info_default() {
        let info = ServerInfo::default();
        assert!(info.id.is_empty());
        assert!(info.name.is_empty());
        assert_eq!(info.port, 0);
        assert_eq!(info.status, ServerStatus::Stopped);
        assert!(info.created_timestamp > 0);
        assert!(!info.created_at.is_empty());
    }

    #[test]
    fn test_server_context_default() {
        let ctx = ServerContext::default();
        let servers = ctx.servers.read().unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_server_context_add_and_read() {
        let ctx = ServerContext::default();
        {
            let mut servers = ctx.servers.write().unwrap();
            servers.insert(
                "test-id".to_string(),
                ServerInfo {
                    id: "test-id".to_string(),
                    name: "testserver".to_string(),
                    port: 8080,
                    status: ServerStatus::Running,
                    ..Default::default()
                },
            );
        }

        let servers = ctx.servers.read().unwrap();
        assert_eq!(servers.len(), 1);
        let server = servers.get("test-id").unwrap();
        assert_eq!(server.name, "testserver");
        assert_eq!(server.status, ServerStatus::Running);
    }
}
