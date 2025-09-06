# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.3.6+`** for a stable release!

**Rush Sync Server** is a professional web server orchestration platform written in Rust. It combines a robust Terminal UI (TUI) with internationalization, theming, a modular command system, production HTTPS/TLS, reverse proxy, WebSocket-based hot reload, and a live dashboard.

**NEW in v0.3.6**: Anti-Flicker color mapping for display labels, hardened terminal lifecycle (TerminalManager + safe restart flow), unified widget/input system with viewported rendering and blinking cursor, multi-terminal cursor coloring (Apple Terminal/iTerm/tmux), minimal dashboard CSS reset, viewport safety guards, and extended logging/i18n.

---

## Project Vision

Rush Sync Server development phases:

- **Phase 0** ✅: Terminal UI foundation with command system
- **Phase 1** ✅ **COMPLETE**: Production-ready server orchestration with enterprise features
- **Phase 2**: Advanced automation & centralized management dashboard
- **Phase 3**: Redis clustering & distributed communication
- **Phase 4**: AI-powered monitoring & predictive scaling

---

## What's New in v0.3.6

### **🖍 Anti-Flicker Display Colors**

A pre-compiled **DISPLAY → Color** mapping eliminates per-frame lookups and flicker when rendering label texts (e.g., `ERROR`, `DEBUG`, `THEME`, `VERSION`). Includes helpers like `available_display_texts()` and ANSI-aware color conversion for the renderer.

### **🖥 Terminal Stability & Safe Restart**

A new `TerminalManager` coordinates raw-mode setup/cleanup with emergency destructors. `restart` now performs a full-screen re-init (terminal, input state, message area) after a confirm prompt, with `--force` bypass support.

### **⌨️ Unified Widget/Input System**

`InputState` implements `Widget`, `CursorWidget`, `StatefulWidget`, and `AnimatedWidget`. Text input renders through a **viewport** (no overflow), with a **blinking cursor** and selection-safe drawing logic. Widgets are easier to compose/test and reuse across app screens.

### **🎯 Cursor Styling & Cross-Terminal Coloring**

Cursor shapes: **PIPE**, **BLOCK**, **UNDERSCORE**. Optional **RGB cursor color** across terminals (Apple Terminal, iTerm, tmux) with graceful fallbacks if true-color is not supported.

### **🧰 System Commands w/ Confirmation**

Centralized internal command processor adds `__CLEAR__`, `__EXIT__`, `__RESTART__`, `__CLEAR_HISTORY__` with structured confirm prompts (incl. cleanup actions). Exposed user commands: `clear`, `restart [-f|--force]`.

### **🗺 Viewport Safety & Render Guards**

Safer layout math and bounds checks to avoid rendering outside terminal area; improved messaging for tiny terminals; emergency fallback render for extreme cases.

### **🖼 Dashboard UX & Minimal CSS Reset**

The aggressive global reset was replaced by a **minimal reset**; dashboard styles were tuned for consistency and resilience. A graceful **server-shutdown page** was added. Monitoring can be paused/resumed; real logs integrate more cleanly.

### **📝 Logging & i18n Extensions**

The server logger uses rotation configuration derived from `LoggingConfig`. Numerous i18n strings were added for screen/theme/viewport/restart diagnostics and user feedback.

---

## 🚀 Production-Ready Server Infrastructure (recap from v0.3.5)

Version 0.3.5 introduced the complete production platform:

- **🔐 Enterprise HTTPS/TLS** — Automatic certificate generation with RSA‑2048 and wildcard/SAN support
- **🌍 Reverse Proxy System** — nginx‑style proxy with SSL termination on port 8443
- **⚡ Hot Reload Development** — Real-time file watching with WebSocket-based browser refresh
- **🛡️ Advanced Security Suite** — Intrusion detection, rate limiting, and audit logging
- **📊 Live Dashboard Interface** — Professional web UI with metrics, logs, TLS management
- **🔄 Intelligent Performance** — Optimized middleware pipeline for faster request handling

---

## 🔐 Advanced HTTPS/TLS System

**Automatic Certificate Management:**

- **Self-Signed Certificates** — RSA‑2048 encryption with 365‑day validity
- **Wildcard Support** — `*.localhost` certificates for seamless subdomain routing
- **Subject Alternative Names** — Multi-domain support (localhost, 127.0.0.1, custom domains)
- **Auto-Generation** — Certificates created on-demand per server
- **Secure Key Storage** — `0600` permissions on private keys with organized directories

**Certificate Structure:**

```bash
.rss/certs/myserver-8080.cert    # Server-specific certificate
.rss/certs/myserver-8080.key     # Private key (0600)
.rss/certs/proxy-8443.cert       # Proxy wildcard certificate
.rss/certs/proxy-8443.key        # Proxy private key
```

**Sample Details:**

```code
Common Name: myserver.localhost
Subject Alt Names: localhost, 127.0.0.1, myserver.localhost
Key Type: RSA-2048
Validity: 365 days
Organization: Rush Sync Server
```

---

## 🌍 Professional Reverse Proxy

**Enterprise-Grade Features:**

- **SSL Termination** — HTTPS proxy on :8443 with automatic certificates
- **Dynamic Routing** — Subdomain routing (e.g., api.localhost → 127.0.0.1:8080)
- **Load Balancing** — Round-robin across multiple instances
- **Health Checks** — Upstream monitoring with failover
- **Request Rewriting** — Header injection and path manipulation

**Proxy Usage Example:**

```bash
# Start servers
create api 8080
create admin 8081

# Access via proxy (automatic HTTPS)
https://api.localhost:8443      # → 127.0.0.1:8080
https://admin.localhost:8443    # → 127.0.0.1:8081

# Optional hosts entries for clarity
127.0.0.1 api.localhost
127.0.0.1 admin.localhost
```

---

## ⚡ Hot Reload Development System

**Real-Time Development:**

- **File Watching** — HTML, CSS, JS, JSON, SVG, images
- **WebSocket Integration** — Instant browser refresh
- **Intelligent Filtering** — Ignores temp/hidden files
- **Debounced Reloading** — Prevents duplicate refreshes
- **Dev Notifications** — Visual change feedback

**Injection & Endpoint:**

```html
<script src="/rss.js"></script>
<link rel="stylesheet" href="/.rss/global-reset.css" />
```

```code
ws://127.0.0.1:8080/ws/hot-reload
```

**Event Example:**

```json
{
  "event_type": "modified",
  "file_path": "www/myserver-[8080]/index.html",
  "server_name": "myserver",
  "port": 8080,
  "timestamp": 1703875457,
  "file_extension": "html"
}
```

---

## 📊 Professional Dashboard Interface

**Comprehensive Management UI:**

- **Live Overview** — Status, metrics, performance
- **Interactive API Testing** — Inline request/response
- **Live Log Viewer** — Streaming with filters
- **TLS Manager** — Certificate status and renewal info
- **Hot Reload Monitor** — WebSocket status & file changes
- **Performance Metrics** — Response times, error rates, traffic

**Endpoints:**

```bash
http://127.0.0.1:8080/.rss/         # Main dashboard
http://127.0.0.1:8080/api/status    # Server status API
http://127.0.0.1:8080/api/metrics   # Performance metrics
http://127.0.0.1:8080/api/logs/raw  # Live log stream
http://127.0.0.1:8080/ws/hot-reload # WebSocket hot reload
```

---

## 🛡️ Enterprise Security Suite

**Monitoring & Protections:**

- **Intrusion Detection** — Detects traversal, XSS, SSRF patterns
- **Request Size Limits** — Prevent simple DoS via large bodies
- **Suspicious Pattern Detection** — Header/path analysis
- **Security Audit Logging** — Detailed, structured logs
- **Rate Limiting** — Per-IP throttling with thresholds

**Security Event Format:**

```json
{
  "event_type": "SecurityAlert",
  "ip_address": "192.168.1.100",
  "alert_reason": "Path Traversal Attempt",
  "alert_details": "Path contains '../' sequence: /../../etc/passwd",
  "timestamp": "2025-01-20 14:30:25.123",
  "headers": {
    "user-agent": "Mozilla/5.0...",
    "referer": "http://malicious-site.com"
  }
}
```

---

## 🎯 Performance Optimizations

- **Optimized Middleware** — Reduced overhead
- **Efficient Memory** — Buffer reuse and allocation trims
- **Concurrency** — Tuned worker pool
- **Intelligent Caching** — Static asset cache headers
- **DB Connection Pooling** — Efficient registry access

---

## 🚀 Installation & Usage

### 📦 **As Binary — Version 0.3.6+**

```bash
# Install from crates.io
cargo install rush-sync-server

# Run with full production features
rush-sync
```

### 📚 **As Library — Version 0.3.6+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.3.6"
tokio = { version = "1.36", features = ["full"] }
```

#### **Quick Start Examples:**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Run with full production features (HTTPS, Proxy, Hot Reload)
    run().await?;
    Ok(())
}
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Custom configuration with enhanced security
    let mut config = load_config().await?;
    config.server.enable_https = true;
    config.proxy.enabled = true;
    config.logging.log_security_alerts = true;

    run_with_config(config).await?;
    Ok(())
}
```

---

## 🌟 Enterprise Features (v0.3.6)

The configuration surface remains consistent with v0.3.5. The sections below summarize the production-related knobs and the new **terminal/UI** aspects of v0.3.6.

### **🔐 HTTPS/TLS Configuration**

```toml
# rush.toml
[server]
enable_https = true
auto_cert = true
cert_dir = ".rss/certs"
cert_validity_days = 365
https_port_offset = 1000

[proxy]
enabled = true
port = 8443
ssl_termination = true
health_check_interval = 30
```

### **🌍 Reverse Proxy System**

```toml
[proxy]
enabled = true
port = 8443
max_connections = 1000
timeout_seconds = 30
buffer_size_kb = 64
worker_threads = 4

[proxy.health_check]
enabled = true
interval_seconds = 30
timeout_seconds = 5
unhealthy_threshold = 3
healthy_threshold = 2
```

**Dynamic Routing Examples:**

```bash
# Create multiple servers
create api 8080
create admin 8081
create docs 8082

# Access via proxy (automatic HTTPS + routing)
https://api.localhost:8443    → 127.0.0.1:8080
https://admin.localhost:8443  → 127.0.0.1:8081
https://docs.localhost:8443   → 127.0.0.1:8082
```

### **⚡ Hot Reload Development**

```toml
[development]
hot_reload = true
watch_extensions = ["html", "css", "js", "json", "svg", "png", "jpg", "ico"]
ignore_patterns = ["*.tmp", "*.swp", ".*", "*~"]
debounce_ms = 250
auto_refresh_browser = true

[development.notifications]
enabled = true
duration_ms = 3000
position = "top-right"
```

### **📊 Advanced Logging System**

```toml
[logging]
max_file_size_mb = 100          # Log rotation size
max_archive_files = 9           # Number of compressed archives
compress_archives = true        # GZIP compressed archives
log_requests = true
log_security_alerts = true
log_performance = true
log_format = "json"
```

**Log Entry Structure:**

```json
{
  "timestamp": "2025-01-20 14:30:25.123",
  "timestamp_unix": 1705757425,
  "event_type": "Request",
  "ip_address": "127.0.0.1",
  "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)...",
  "method": "GET",
  "path": "/api/status",
  "status_code": 200,
  "response_time_ms": 15,
  "bytes_sent": 1024,
  "referer": "https://myserver.localhost:8443/",
  "headers": {
    "accept": "application/json",
    "host": "myserver.localhost:8443",
    "x-forwarded-for": "127.0.0.1",
    "x-forwarded-proto": "https"
  },
  "session_id": "sess_abc123",
  "tls_version": "TLSv1.3",
  "cipher_suite": "TLS_AES_256_GCM_SHA384"
}
```

---

## 💻 Enhanced Server Management Commands

### **🌍 Production Server Commands**

| Command   | Description                     | Examples                                   |
| --------- | ------------------------------- | ------------------------------------------ |
| `create`  | Create server with HTTPS/TLS    | `create`, `create api`, `create docs 8090` |
| `list`    | Show servers with TLS status    | `list`                                     |
| `start`   | Start with proxy registration   | `start 1`, `start api`, `start abc123`     |
| `stop`    | Stop with proxy cleanup         | `stop 1`, `stop api`, `stop abc123`        |
| `cleanup` | Remove servers and certificates | `cleanup`, `cleanup failed`, `cleanup all` |

### **🔐 TLS Management Commands**

| Command        | Description                 | Examples              |
| -------------- | --------------------------- | --------------------- |
| `cert list`    | Show all certificates       | `cert list`           |
| `cert info`    | Certificate details         | `cert info myserver`  |
| `cert renew`   | Regenerate certificate      | `cert renew myserver` |
| `cert cleanup` | Remove expired certificates | `cert cleanup`        |

### **🌍 Proxy Management Commands**

| Command        | Description                | Examples       |
| -------------- | -------------------------- | -------------- |
| `proxy status` | Show proxy status          | `proxy status` |
| `proxy routes` | List all proxy routes      | `proxy routes` |
| `proxy start`  | Start proxy server         | `proxy start`  |
| `proxy stop`   | Stop proxy server          | `proxy stop`   |
| `proxy reload` | Reload proxy configuration | `proxy reload` |

### **⚡ Development Commands**

| Command        | Description               | Examples              |
| -------------- | ------------------------- | --------------------- |
| `dev mode on`  | Enable development mode   | `dev mode on`         |
| `dev mode off` | Disable development mode  | `dev mode off`        |
| `dev watch`    | Show file watching status | `dev watch`           |
| `dev reload`   | Trigger manual reload     | `dev reload myserver` |

### **🧰 New System Commands (v0.3.6)**

| Command   | Description             | Examples                |
| --------- | ----------------------- | ----------------------- |
| `restart` | Restart the application | `restart`, `restart -f` |
| `clear`   | Clear the screen        | `clear`, `cls`          |

---

## 📊 Advanced Server Examples

### **🚀 Production Server Deployment**

```bash
# Create production API server with HTTPS
create api 8080
# Result: Server created: 'api' (ID: abc12345) on Port 8080
# HTTPS: https://api.localhost:8443 (via proxy)
# HTTP: http://127.0.0.1:8080 (direct)
# Certificate: .rss/certs/api-8080.cert
# Hot Reload: WebSocket on ws://127.0.0.1:8080/ws/hot-reload

# Enhanced server list with production details
list
# Result:
# Server List (Production Mode - Max: 10 concurrent):
#   1. api - abc12345 (Port: 8080) [Running] 🔒 HTTPS
#      URLs: https://api.localhost:8443 | http://127.0.0.1:8080
#      Certificate: Valid (362 days remaining)
#      Hot Reload: Active | Proxy: Registered
#      Log: .rss/servers/api-[8080].log (23.4MB, 2 archives)
#      Requests: 5,847 | Errors: 12 | Security Alerts: 0
#      Avg Response: 18ms | Uptime: 4h 32m
#
#   2. admin - def67890 (Port: 8081) [Running] 🔒 HTTPS
#      URLs: https://admin.localhost:8443 | http://127.0.0.1:8081
#      Certificate: Valid (364 days remaining)
#      Hot Reload: Active | Proxy: Registered
```

### **📊 Advanced Monitoring & Statistics**

```bash
curl https://api.localhost:8443/api/metrics
{
  "server_info": {
    "id": "abc12345",
    "name": "api",
    "port": 8080,
    "status": "running",
    "uptime_seconds": 16320,
    "version": "0.3.6"
  },
  "security": {
    "tls_enabled": true,
    "certificate_valid": true,
    "certificate_expires": "2025-12-31T23:59:59Z",
    "security_alerts_24h": 0,
    "blocked_ips": []
  },
  "performance": {
    "total_requests": 5847,
    "requests_per_second": 1.2,
    "avg_response_time_ms": 18,
    "max_response_time_ms": 245,
    "error_rate_percent": 0.21
  },
  "proxy": {
    "registered": true,
    "health_check_status": "healthy",
    "last_health_check": "2025-01-20T14:29:55Z",
    "proxy_requests": 4203,
    "direct_requests": 1644
  },
  "hot_reload": {
    "enabled": true,
    "websocket_connections": 2,
    "file_changes_24h": 47,
    "last_reload": "2025-01-20T13:15:32Z"
  },
  "logging": {
    "log_file_size_mb": 23.4,
    "archive_count": 2,
    "log_entries_24h": 5847,
    "security_events_24h": 0,
    "error_events_24h": 12
  }
}
```

### **🔐 TLS Certificate Management**

```bash
# View all certificates
cert list
# Result:
# TLS Certificate List:
#   api-8080.cert
#     Common Name: api.localhost
#     Valid Until: 2025-12-31 (362 days)
#     Key Type: RSA-2048
#     File Size: 1.2KB
#
#   proxy-8443.cert
#     Common Name: *.localhost (Wildcard)
#     Valid Until: 2025-12-31 (364 days)
#     Key Type: RSA-2048
#     File Size: 1.3KB

# Detailed certificate information
cert info api
# Result:
# Certificate Details: api-8080.cert
# ====================================
# Subject: CN=api.localhost, O=Rush Sync Server
# Issuer: CN=api.localhost, O=Rush Sync Server (Self-Signed)
# Valid From: 2025-01-20 00:00:00 UTC
# Valid Until: 2025-12-31 23:59:59 UTC (362 days remaining)
# Serial Number: 1a:2b:3c:4d:5e:6f
# Key Algorithm: RSA-2048
# Signature Algorithm: SHA256-RSA
# Subject Alt Names:
#   - DNS: localhost
#   - DNS: api.localhost
#   - IP: 127.0.0.1
# Certificate File: .rss/certs/api-8080.cert (1,247 bytes)
# Private Key File: .rss/certs/api-8080.key (1,679 bytes, 0600)
```

---

## ⚙️ Production Configuration

### 📁 File Structure

```bash
.rss/
├── rush.toml                    # Main configuration
├── rush.history                 # Command history
├── rush.logs                    # Application logs
├── servers.list                 # Server registry
├── certs/                       # TLS certificates
│   ├── api-8080.cert
│   ├── api-8080.key             # Private keys (0600)
│   ├── proxy-8443.cert
│   └── proxy-8443.key
├── servers/                     # Individual server logs
│   ├── api-[8080].log           # Current log file
│   ├── api-[8080].1.log.gz      # Compressed archive
│   └── api-[8080].2.log.gz      # Older archives
└── proxy/                       # Proxy configuration
    ├── routes.json              # Dynamic routing table
    ├── health_checks.json       # Health check results
    └── access.log               # Proxy access logs
```

### 🛠 Complete Configuration (v0.3.6)

```toml
[general]
max_messages = 1000
typewriter_delay = 5
input_max_length = 100
max_history = 30
poll_rate = 16
log_level = "info"
current_theme = "dark"

[language]
current = "en"

# Server Configuration
[server]
port_range_start = 8080
port_range_end = 8180
max_concurrent = 10
shutdown_timeout = 5
startup_delay_ms = 500
workers = 1

# HTTPS/TLS Configuration
enable_https = true
auto_cert = true
cert_dir = ".rss/certs"
cert_validity_days = 365
https_port_offset = 1000

# Reverse Proxy Configuration
[proxy]
enabled = true
port = 8443
max_connections = 1000
timeout_seconds = 30
buffer_size_kb = 64
worker_threads = 4
ssl_termination = true

[proxy.health_check]
enabled = true
interval_seconds = 30
timeout_seconds = 5
unhealthy_threshold = 3
healthy_threshold = 2

# Advanced Logging
[logging]
max_file_size_mb = 100
max_archive_files = 9
compress_archives = true
log_requests = true
log_security_alerts = true
log_performance = true
log_format = "json"

# Development
[development]
hot_reload = true
watch_extensions = ["html", "css", "js", "json", "svg", "png", "jpg", "ico"]
ignore_patterns = ["*.tmp", "*.swp", ".*", "*~"]
debounce_ms = 250
auto_refresh_browser = true

[development.notifications]
enabled = true
duration_ms = 3000
position = "top-right"

# Security
[security]
max_request_size_mb = 10
rate_limit_requests_per_minute = 60
enable_intrusion_detection = true
log_security_events = true
block_suspicious_ips = false

# Theme
[theme.dark]
output_bg = "Black"
output_text = "White"
output_cursor = "PIPE"
output_cursor_color = "White"
input_bg = "White"
input_text = "Black"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "Black"
```

---

## 🧪 Quality Assurance & Testing

### 📈 Performance Benchmarks (v0.3.5 baseline)

```bash
Server Creation: ~300ms (40% faster than v0.3.3)
TLS Certificate Generation: ~150ms per certificate
Proxy Route Registration: ~50ms per server
Hot Reload WebSocket Setup: ~25ms

Concurrent Users: 1000+ users per server
Request Throughput: 5000+ requests/second
Memory Usage: <50MB per server instance
CPU Usage: <5% under normal load

HTTPS Handshake: ~15ms average
Certificate Validation: ~2ms average
SSL Termination Overhead: <5% vs HTTP
```

## _(v0.3.6 does not change these baseline numbers; focus was stability, UI/UX, and DX.)_

### 🛡️ Testing Suite

```bash
# Core functionality tests
cargo test server_lifecycle_with_tls
cargo test proxy_routing_and_ssl
cargo test hot_reload_websocket
cargo test security_monitoring
cargo test certificate_management

# Load and stress testing
cargo test --release concurrent_https_servers
cargo test --release proxy_load_balancing
cargo test --release tls_performance_under_load
cargo test --release hot_reload_stress_test

# Security testing
cargo test intrusion_detection_patterns
cargo test rate_limiting_enforcement
cargo test certificate_validation
cargo test suspicious_request_blocking
```

---

## 📊 Version History

### **v0.3.6 (Current) — UI/Terminal & DX Enhancements**

- **Anti-Flicker Color System** for display labels (zero-delay color mapping).
- **TerminalManager** with raw-mode tracking, safe cleanup, emergency destructor.
- **Safe Restart Flow** (`restart`, confirm prompts, re-init of terminal & UI).
- **Widget/Input Unification** with viewported rendering & blinking cursor.
- **Cursor Styling** (PIPE/BLOCK/UNDERSCORE + RGB across terminals/tmux).
- **Dashboard UX**: minimal reset CSS, shutdown screen, improved monitoring.
- **Logging**: Server logger API w/ rotation config; **i18n** keys expanded.

### **v0.3.5 — Production Infrastructure**

- Complete HTTPS/TLS system, enterprise reverse proxy with SSL termination, advanced hot reload, security monitoring suite, professional dashboard, and performance pipeline optimizations.

### **v0.3.3 — Optimized Architecture & Logging**

- **35% Code Reduction** while preserving functionality
- **Structured Logging** with rotation and compression
- **Performance Improvements** (~40% faster request processing)

### **v0.3.2 — Complete Server Management**

- **Actix-Web Integration**: production web server creation and management
- **Dynamic Server Lifecycle**: full orchestration capabilities

---

## 🏆 Code Quality Metrics (v0.3.6)

- ✅ **Zero Cargo errors** on full feature set
- ✅ **Hardened terminal lifecycle** (raw-mode detection, emergency cleanup)
- ✅ **UI stability** via viewport checks & anti-flicker colors
- ✅ **Thread/Memory Safety** (Rust guarantees; async-safe state)
- ✅ **Enterprise Logging** (structured JSON + rotation)
- ✅ **Performance-Optimized** (no regressions vs 0.3.5)
- ✅ **Comprehensive Testing** (incl. TLS, proxy, security)
- ✅ **Professional UI** (modern dashboard with live metrics & TLS status)
- ✅ **Cross-Platform** (macOS, Linux, Windows)

## **Security**

- TLS 1.3 with modern cipher suites
- Proper certificate validation
- Intrusion detection & rate limiting
- Security audit logging

---

## 📜 License

### Dual-Licensing Model

1. **Community License (GPLv3)** — Free for private and non-commercial use
2. **Commercial License** — Required for commercial applications and enterprise deployments

**Commercial licensing inquiries:**
📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)

---

## 🤝 Contributing

**Phase 2 Targets:**

- Advanced load balancing with health-aware routing
- Docker/Kubernetes integration
- Real-time analytics dashboard
- Centralized configuration across clusters

**Security Enhancements:**

- Let’s Encrypt integration
- Sliding-window rate limiting
- WAF integration
- OAuth2/JWT auth

**Performance & Scalability:**

- Redis-based sessions/caching
- DB connection pooling
- CDN for static assets
- Auto-scaling triggers

**Development Guidelines:**

- Keep **clippy** clean; comprehensive lints
- Tests for every security-sensitive feature
- Async/await best practices
- Error handling with context (anyhow/thiserror)

---

## 📞 Contact & Support

- **Primary Contact:** 📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)
- **GitHub Repository:** [LEVOGNE/rush.sync.server](https://github.com/LEVOGNE/rush.sync.server)
- **Issues & Bug Reports:** GitHub Issues
- **Feature Requests:** GitHub Discussions
- **Security Issues:** 📧 [security@rush-sync.dev](mailto:security@rush-sync.dev)

---

_Rush Sync Server v0.3.6 — Production-grade orchestration with hardened terminal lifecycle, anti-flicker UI, safe restart flow, minimal CSS reset, live dashboard, and comprehensive security/monitoring._
