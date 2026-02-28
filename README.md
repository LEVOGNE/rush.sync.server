# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.83+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Tests](https://img.shields.io/badge/tests-120%20passing-brightgreen)
![Clippy](https://img.shields.io/badge/clippy-zero%20warnings-brightgreen)
![Docker](https://img.shields.io/badge/docker-ready-blue)
![License](https://img.shields.io/badge/license-Dual--License-blue)

**Rush Sync Server** is a multi-server orchestration platform written in Rust. It combines a terminal UI (TUI) with automatic HTTPS/TLS, a reverse proxy with subdomain routing, WebSocket-based hot reload, structured logging, and a live dashboard — all managed through an interactive command interface.

One binary. Zero external dependencies. No nginx, no caddy, no certbot.

```
Internet
  │
  ├── default.example.com ──┐
  ├── myapp.example.com ────┤
  ├── blog.example.com ─────┤
  │                          ▼
  │                 ┌─────────────────┐
  │                 │  Reverse Proxy  │
  │                 │  :80 / :443     │
  │                 └──┬────┬────┬────┘
  │                    ▼    ▼    ▼
  │                 :8000 :8001 :8002
  │                 default myapp blog
```

---

## Features

- **Multi-Server Management** — Create, start, stop, and monitor up to 50 web servers from a single instance
- **Production Ready** — Configurable bind address (`0.0.0.0`), real domain support, headless/daemon mode
- **Automatic HTTPS/TLS** — Dual HTTP+HTTPS binding per server, self-signed certificates with production domain SANs
- **Let's Encrypt Integration** — Automatic ACME certificate provisioning with HTTP-01 challenges and background renewal
- **File Upload API** — Upload, list, and delete website files via REST API — no `scp` or `rsync` needed
- **Reverse Proxy** — Subdomain-based routing (`myapp.example.com` -> backend port)
- **Headless Mode** — Run without terminal via `--headless` / `--daemon` with auto-start of marked servers
- **Docker Ready** — Multi-stage Dockerfile, docker-compose, automatic config generation
- **Hot Reload** — File watching with WebSocket-based browser refresh
- **Live Dashboard** — Web UI with server status, metrics, and API documentation
- **API-Key Authentication** — HMAC-SHA256 hashed keys, `.env`/env-var support, timing-safe comparison
- **Rate Limiting** — Per-IP sliding-window rate limiter for API endpoints (configurable RPS)
- **Security Middleware** — Path traversal detection, XSS prevention, SQL injection detection
- **Structured Logging** — JSON log files with rotation and compression
- **Internationalization** — Multi-language TUI with i18n support (English, German)
- **Theming** — Configurable colors, cursor styles, and input prefixes

---

## Quick Start

### Docker (recommended)

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
docker compose up
```

This will:
1. Compile the Rust binary on Linux (multi-stage build)
2. Generate a Docker-optimized `rush.toml` with `bind_address = "0.0.0.0"`
3. Create a default server on port 8000 with auto-start
4. Start the reverse proxy on ports 3000 (HTTP) and 3443 (HTTPS)

Access: `http://localhost:8000`

### Install from crates.io

```bash
cargo install rush-sync-server
rush-sync
```

### Run from source

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
cargo run
```

### Headless / Daemon Mode

```bash
# Run without terminal (for Linux servers, systemd, Docker)
rush-sync --headless

# Or from source:
cargo run -- --headless
```

The headless mode auto-starts servers marked for auto-start, initializes the reverse proxy, and waits for `SIGINT`/`SIGTERM` for graceful shutdown.

### Use as a library

```toml
[dependencies]
rush-sync-server = "0.3.8"
tokio = { version = "1.36", features = ["full"] }
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
```

---

## Docker Deployment

### Files

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build: `rust:1.83-bookworm` (builder) → `debian:bookworm-slim` (runtime) |
| `docker-compose.yml` | Service definition with ports, volumes, env |
| `docker-entrypoint.sh` | Auto-generates config, seeds default server, starts headless |
| `.env.docker` | Example env with `RSS_API_KEY` |
| `.dockerignore` | Excludes `target/`, `.git/`, etc. |

### Architecture

```
┌─────────────────────────────────────────────┐
│  Docker Container                           │
│                                             │
│  docker-entrypoint.sh                       │
│    ├── Generates /app/.rss/rush.toml        │
│    ├── Seeds default server (auto-start)    │
│    └── exec rush-sync --headless            │
│                                             │
│  Ports:                                     │
│    8000  ─── HTTP Server (default)          │
│    8001  ─── HTTP Server (optional)         │
│    3000  ─── Reverse Proxy HTTP             │
│    3443  ─── Reverse Proxy HTTPS            │
│                                             │
│  Volumes:                                   │
│    /app/.rss  ─── Config, certs, logs       │
│    /app/www   ─── Website files             │
└─────────────────────────────────────────────┘
```

### Commands

```bash
# Build and start
docker compose up

# Build and start (detached)
docker compose up -d

# View logs
docker compose logs -f

# Stop
docker compose down

# Full reset (removes config + data)
docker compose down -v

# Hash an API key
docker compose run --rm --entrypoint /app/rush-sync rush-sync --hash-key my-secret-key

# Shell into container
docker compose exec rush-sync sh
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RSS_API_KEY` | API key for authentication (overrides `rush.toml`) |

---

## Production Deployment

Rush can host real domains on a public server without a reverse proxy like nginx.

### 1. Configure `rush.toml`

```toml
[server]
bind_address = "0.0.0.0"           # Listen on all interfaces
port_range_start = 8000
port_range_end = 8999              # Increase range for production
max_concurrent = 200
production_domain = "example.com"  # Your real domain
api_key = "$hmac-sha256$..."       # Hash via: rush-sync --hash-key <your-key>
rate_limit_rps = 100               # API rate limiting

# Let's Encrypt (automatic HTTPS certificates)
use_lets_encrypt = true            # Enable automatic certificate provisioning
acme_email = "admin@example.com"   # Notification email (optional)

[proxy]
enabled = true
port = 80                          # Standard HTTP (required for Let's Encrypt)
https_port_offset = 443            # HTTPS on port 443+80 = 523 (or set port=443)
bind_address = "0.0.0.0"           # Public proxy
```

> **Important:** When using `bind_address = "0.0.0.0"`, always set `api_key` (or `RSS_API_KEY` env var) to prevent unauthorized access to management endpoints. Use `rush-sync --hash-key <your-key>` to avoid storing plaintext keys.
>
> **Let's Encrypt:** Requires the proxy on port 80 and DNS pointing to your server. Certificates are provisioned automatically on startup and renewed 30 days before expiry.

### 2. DNS Setup

Point a wildcard DNS record to your server:

```
*.example.com  A  <your-server-ip>
example.com    A  <your-server-ip>
```

### 3. Run

```bash
# With Docker (recommended):
docker compose up -d

# Or directly with sudo for ports 80/443:
sudo rush-sync --headless

# Or use higher ports without sudo:
rush-sync --headless
```

### 4. Home Server (Fritz!Box example)

You can also run rush-sync-server at home behind a router:

```
# Fritz!Box Port Forwarding:
Port 80  (extern) → 192.168.x.x:3000  (Proxy HTTP)
Port 443 (extern) → 192.168.x.x:3443  (Proxy HTTPS)

# DNS at your domain provider:
example.com    A    <your-public-ip>
*.example.com  A    <your-public-ip>
```

Enable DynDNS in your Fritz!Box if your ISP assigns dynamic IPs.

### 5. Access

```
http://myapp.example.com   ->  backend on auto-assigned port
http://api.example.com     ->  backend on auto-assigned port
```

---

## Commands

| Command    | Description                          | Examples                                    |
|------------|--------------------------------------|---------------------------------------------|
| `create`   | Create a new web server              | `create`, `create api`, `create docs 8090`  |
| `start`    | Start server(s)                      | `start 1`, `start api`, `start 1-100`, `start all` |
| `stop`     | Stop server(s)                       | `stop 1`, `stop api`, `stop 1-50`, `stop all` |
| `list`     | Show all servers with status         | `list`                                      |
| `cleanup`  | Remove stopped/failed servers        | `cleanup`, `cleanup all`                    |
| `recovery` | Recover servers from disk            | `recovery`                                  |
| `restart`  | Restart the TUI application          | `restart`, `restart -f`                     |
| `clear`    | Clear the screen                     | `clear`, `cls`                              |
| `history`  | Show command history                 | `history`                                   |
| `remote`   | Manage SSH remote profiles           | `remote add prod user@host /opt/app`        |
| `sync`     | Push/pull/exec over SSH              | `sync push prod ./www`, `sync exec prod uptime` |
| `theme`    | Change the UI theme                  | `theme dark`, `theme light`                 |
| `lang`     | Change language                      | `lang en`, `lang de`                        |
| `loglevel` | Change log verbosity                 | `loglevel debug`, `loglevel info`           |
| `version`  | Show version info                    | `version`, `ver`                            |
| `help`     | Show available commands              | `help`                                      |
| `exit`     | Exit the application                 | `exit`, `quit`                              |

Bulk operations support ranges up to 500 servers (e.g. `start 1-200`).

With the `memory` feature enabled:

| Command | Description                          | Examples                        |
|---------|--------------------------------------|---------------------------------|
| `mem`   | Memory & process introspection       | `mem info`, `mem info --all`    |

---

## Server Workflow

```bash
# Create a server (auto-assigns port from configured range)
create myapp

# Start it
start myapp

# Access it
# Direct:    http://<bind_address>:8000
# Proxy:     http://myapp.<domain>:<proxy_port>
# Dashboard: http://<bind_address>:8000/.rss/

# Create more servers
create api
create docs 8090

# Bulk operations
start all
start 1-50

# List everything
list

# Stop and clean up
stop all
cleanup
```

---

## API Endpoints

Every server exposes these endpoints. When `api_key` is configured, endpoints marked with a lock require authentication (via `X-API-Key` header or `?api_key=` query parameter):

| Endpoint              | Method | Auth | Description                     |
|-----------------------|--------|------|---------------------------------|
| `/`                   | GET    |      | Static files / index.html       |
| `/api/health`         | GET    |      | Health check (always public)    |
| `/.well-known/acme-challenge/{token}` | GET | | Let's Encrypt HTTP-01 challenge |
| `/.rss/`              | GET    | \*   | Live dashboard                  |
| `/api/status`         | GET    | \*   | Server status and configuration |
| `/api/info`           | GET    | \*   | API documentation               |
| `/api/metrics`        | GET    | \*   | Performance metrics             |
| `/api/stats`          | GET    | \*   | Request statistics              |
| `/api/logs`           | GET    | \*   | Log viewer (HTML)               |
| `/api/logs/raw`       | GET    | \*   | Log data (JSON, incremental)    |
| `/api/ping`           | POST   | \*   | Ping/pong echo                  |
| `/api/message`        | POST   | \*   | Send a message                  |
| `/api/messages`       | GET    | \*   | Retrieve stored messages        |
| `/api/files`          | GET    | \*   | List files in server directory  |
| `/api/files/{path}`   | PUT    | \*   | Upload/create a file            |
| `/api/files/{path}`   | DELETE | \*   | Delete a file or directory      |
| `/ws/hot-reload`      | WS     | \*   | WebSocket file change events    |

\* Protected when `api_key` is set. Returns `401 Unauthorized` without valid key.

### File Upload API

Deploy website files remotely without `scp` or `rsync`:

```bash
# Upload a file
curl -X PUT -H "X-API-Key: your-key" \
  --data-binary @index.html \
  http://localhost:8080/api/files/index.html

# Upload to subdirectory
curl -X PUT -H "X-API-Key: your-key" \
  --data-binary @logo.png \
  http://localhost:8080/api/files/images/logo.png

# List files
curl -H "X-API-Key: your-key" http://localhost:8080/api/files

# List subdirectory
curl -H "X-API-Key: your-key" "http://localhost:8080/api/files?path=images"

# Delete a file
curl -X DELETE -H "X-API-Key: your-key" \
  http://localhost:8080/api/files/old-page.html
```

Subdirectories are created automatically. Path traversal is blocked.

---

## Reverse Proxy

The integrated reverse proxy maps subdomains to server ports:

```
myapp.example.com:80   ->  127.0.0.1:8080
api.example.com:80     ->  127.0.0.1:8081
docs.example.com:80    ->  127.0.0.1:8082
```

Works with any domain — `localhost` for development, real domains for production. Routes are registered automatically when a server starts and removed when it stops. The proxy supports both HTTP and HTTPS with automatic TLS certificate generation.

CORS is automatically configured to allow your `production_domain` in addition to `localhost`.

---

## Hot Reload

Servers watch their `www/{name}-[{port}]/` directory for file changes. When HTML, CSS, JS, JSON, SVG, or image files change, a WebSocket event is broadcast to all connected browsers:

```json
{
  "event_type": "modified",
  "file_path": "www/myapp-[8080]/index.html",
  "server_name": "myapp",
  "port": 8080,
  "timestamp": 1703875457,
  "file_extension": "html"
}
```

The `rss.js` module is automatically injected into served HTML files and connects to the WebSocket endpoint to trigger page reloads.

---

## HTTPS/TLS

When enabled, each web server binds on both HTTP and HTTPS simultaneously:

```
HTTP:   http://127.0.0.1:8080
HTTPS:  https://127.0.0.1:9080   (port + https_port_offset)
```

Certificates are automatically generated per server:

```
.rss/certs/myapp-8080.cert    # Server certificate
.rss/certs/myapp-8080.key     # Private key
.rss/certs/proxy-3000.cert    # Proxy wildcard certificate
```

Certificates include Subject Alternative Names for:
- `localhost`, `127.0.0.1`, `{name}.localhost` (always)
- `{production_domain}`, `*.{production_domain}`, `{name}.{production_domain}` (when configured)

If the HTTPS bind fails (e.g. port conflict), the server continues with HTTP only.

### Let's Encrypt (Automatic)

Enable automatic certificate provisioning via ACME HTTP-01 challenges:

```toml
[server]
use_lets_encrypt = true
production_domain = "example.com"
acme_email = "admin@example.com"

[proxy]
port = 80    # Required for Let's Encrypt verification
```

On startup, rush.sync.server will:
1. Register an ACME account (key stored in `.rss/certs/acme-account.key`)
2. Request a certificate for `production_domain`
3. Serve the HTTP-01 challenge via the proxy on port 80
4. Download and store the certificate
5. Check for renewal every 24 hours (renews 30 days before expiry)

Certificates are stored as:
```
.rss/certs/example.com.fullchain.pem
.rss/certs/example.com.privkey.pem
```

### Manual Certificates

Alternatively, place certificate files manually in the cert directory:

```
.rss/certs/example.com.fullchain.pem
.rss/certs/example.com.privkey.pem
```

---

## Security

### API-Key Authentication

Management endpoints (`/api/*`, `/.rss/*`, `/ws/*`) can be protected with an API key. When `api_key` is set in `rush.toml`, all requests to these endpoints require authentication — except `/api/health` which always remains public.

All key comparisons are **timing-safe** (via `ring::hmac`), preventing timing side-channel attacks.

#### Three ways to set the key

**1. Plaintext in TOML** (simplest):

```toml
[server]
api_key = "my-secret-key-123"
```

**2. HMAC-SHA256 hash in TOML** (recommended — key never stored in cleartext):

```bash
# Generate hash
rush-sync --hash-key my-secret-key-123
# Output: $hmac-sha256$<base64>

# Paste into rush.toml
```

```toml
[server]
api_key = "$hmac-sha256$aBcDeFgH..."
```

**3. Environment variable** (CI/CD, Docker, `.env` — never written back to TOML):

```bash
# .env file (loaded automatically via dotenvy)
RSS_API_KEY=my-secret-key-123

# Or export directly
export RSS_API_KEY=my-secret-key-123

# Or via docker-compose (.env.docker)
RSS_API_KEY=my-secret-key-123
```

The `RSS_API_KEY` env var overrides whatever is in `rush.toml`. When the config is saved, env-var keys are never persisted — `api_key` stays `""` in TOML.

#### Authentication

Authenticate via header or query parameter:

```bash
# Header (recommended)
curl -H "X-API-Key: my-secret-key-123" http://localhost:8080/api/status

# Query parameter (for WebSocket clients, browser testing)
curl http://localhost:8080/api/status?api_key=my-secret-key-123
```

When `api_key` is empty and no `RSS_API_KEY` is set (default), all endpoints are open — backwards-compatible with existing setups.

### Rate Limiting

Per-IP sliding-window rate limiting protects `/api/*` endpoints from abuse:

```toml
[server]
rate_limit_rps = 100       # Max requests per second per IP
rate_limit_enabled = true  # Toggle rate limiting
```

Exceeding the limit returns `429 Too Many Requests` with a `Retry-After: 1` header.

### Middleware Stack

The full middleware pipeline (in execution order, outermost first):

1. **CORS** — Origin validation (`localhost` + `production_domain`)
2. **Compression** — Response compression
3. **API-Key Auth** — Key validation on management endpoints
4. **Rate Limiter** — Per-IP request throttling on `/api/*`
5. **Logging** — Structured request logging with security alerts

### Detection & Prevention

- **Path Traversal Protection** — Canonicalized path validation with percent-decoding (`%2e%2e` etc.)
- **XSS Prevention** — HTML and JavaScript escaping for all template variables
- **SQL Injection Detection** — Pattern matching for common injection signatures
- **Header Filtering** — Sensitive headers (`authorization`, `cookie`, `x-api-key`) are redacted in logs

Security alerts are logged when suspicious requests are detected:

```json
{
  "event_type": "SecurityAlert",
  "ip_address": "127.0.0.1",
  "alert_reason": "Suspicious Request",
  "alert_details": "Suspicious path: /../../etc/passwd"
}
```

---

## Configuration

Configuration lives in `rush.toml` (auto-generated on first run):

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

[server]
port_range_start = 8000
port_range_end = 8200              # Port range for auto-allocation
max_concurrent = 50                # Up to 50 simultaneous servers (configurable)
shutdown_timeout = 5
startup_delay_ms = 500
workers = 1
auto_open_browser = true
bind_address = "127.0.0.1"         # "0.0.0.0" for public access
enable_https = true
auto_cert = true
cert_dir = ".rss/certs"
cert_validity_days = 365
https_port_offset = 1000
production_domain = "localhost"    # Set to your real domain for production
use_lets_encrypt = false           # Automatic Let's Encrypt certificates
acme_email = ""                    # Email for Let's Encrypt (optional)
api_key = ""                       # Plaintext, $hmac-sha256$... hash, or use RSS_API_KEY env var
rate_limit_rps = 100               # Max requests/sec per IP for /api/*
rate_limit_enabled = true          # Enable rate limiting

[proxy]
enabled = true
port = 3000
https_port_offset = 443
bind_address = "127.0.0.1"        # "0.0.0.0" for public access
health_check_interval = 30
timeout_ms = 5000

[logging]
max_file_size_mb = 100
max_archive_files = 9
compress_archives = true
log_requests = true
log_security_alerts = true
log_performance = true

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

### Key Configuration Options

| Setting | Default | Description |
|---------|---------|-------------|
| `server.bind_address` | `127.0.0.1` | `0.0.0.0` to listen on all interfaces |
| `server.production_domain` | `localhost` | Your real domain (e.g. `example.com`) |
| `server.use_lets_encrypt` | `false` | Automatic Let's Encrypt certificates |
| `server.acme_email` | `""` | Email for Let's Encrypt notifications |
| `server.max_concurrent` | `50` | Maximum simultaneous servers |
| `server.port_range_start` | `8000` | Lower port for auto-allocation |
| `server.port_range_end` | `8200` | Upper port for auto-allocation |
| `server.api_key` | `""` | Plaintext, `$hmac-sha256$...` hash, or `RSS_API_KEY` env var |
| `server.rate_limit_rps` | `100` | Max requests per second per IP on `/api/*` |
| `server.rate_limit_enabled` | `true` | Enable/disable rate limiting |
| `proxy.bind_address` | `127.0.0.1` | `0.0.0.0` for public proxy access |
| `proxy.port` | `3000` | Set to `80` for production / Let's Encrypt |

---

## File Structure

```
.rss/
├── rush.toml                    # Configuration
├── rush.history                 # Command history
├── servers.list                 # Server registry (JSON)
├── certs/                       # TLS certificates
│   ├── myapp-8080.cert
│   ├── myapp-8080.key
│   ├── proxy-3000.cert
│   └── proxy-3000.key
└── servers/                     # Server log files
    ├── myapp-[8080].log
    └── myapp-[8080].1.log.gz    # Rotated + compressed

www/
├── myapp-[8080]/                # Server document root
│   ├── index.html
│   ├── README.md
│   └── robots.txt
└── api-[8081]/
    └── ...
```

---

## Testing

120 tests across unit, integration, and handler layers:

```bash
cargo test                     # Run all 120 tests
cargo test --features memory   # Include memory module tests
cargo clippy -- -D warnings    # Zero warnings
```

### Test Coverage

| Area                    | Tests | What's tested                                              |
|-------------------------|-------|------------------------------------------------------------|
| API Key (crypto)        | 8     | Empty, plaintext match/mismatch, HMAC hash match/mismatch, env/toml roundtrip, hash format |
| Middleware Security     | 16    | percent_decode, path traversal, XSS, SQL injection, encoding |
| JS Escape (XSS)        | 8     | Quotes, backslash, HTML tags, ampersand, XSS payloads      |
| HTML Escape (XSS)      | 7     | Tags, quotes, ampersand, full XSS payloads                 |
| Script Injection        | 6     | CSS/JS insertion position, head/body/fallback, no duplicates |
| API Handlers            | 14    | health, ping, status, info, message, close-browser          |
| Asset Handlers          | 7     | CSS, favicon, fonts (valid/invalid/all 4), JS templates     |
| XSS Prevention          | 1     | Malicious server name in JS template                        |
| Proxy Manager           | 9     | Route add/remove/overwrite, multiple routes, config          |
| Server Types            | 4     | ServerStatus display, ServerInfo defaults, ServerContext      |
| Command System          | 7     | Core commands, registry, metadata, empty/whitespace input    |
| Bulk Parsing            | 6     | Single, all, range, invalid range, name-with-dash            |
| Config & i18n           | 5     | Default values, translations, available languages            |
| Cursor & Widget         | 4     | Cursor types, position, color, widget system                 |

---

## Feature Flags

```toml
[features]
default = []
memory = ["dep:sysinfo"]    # Memory introspection (mem command, process metrics)
scss = ["sass-rs"]           # SCSS compilation for themes
```

Build with memory support:

```bash
cargo build --features memory
```

---

## Architecture

```
src/
├── main.rs             # Entry point: TUI mode or --headless daemon mode
├── lib.rs              # Library exports
├── bootstrap.rs        # Application bootstrap and initialization
│
├── commands/           # Command system (16 commands)
│   ├── handler.rs      # Input parsing and command dispatch
│   ├── registry.rs     # Command registry
│   ├── command.rs      # Command trait definition
│   ├── parsing.rs      # Shared parsing utilities (BulkMode, ranges up to 500)
│   └── */command.rs    # Individual command implementations
│                       #   cleanup, clear, create, exit, help, history,
│                       #   lang, list, log_level, memory, recovery,
│                       #   remote, restart, start, stop, sync, theme, version
├── core/
│   ├── api_key.rs      # Opaque API key type (HMAC-SHA256 hash, timing-safe verify, env override)
│   ├── config.rs       # TOML configuration loader (ServerConfig, ProxyConfig)
│   ├── constants.rs    # System constants and signal strings
│   ├── error.rs        # Error types (AppError, Result)
│   ├── helpers.rs      # Lock helpers, config loader, base_dir (OnceLock), html_escape
│   └── prelude.rs      # Common imports
│
├── i18n/               # Internationalization (en, de)
├── embedded/           # Embedded static resources
├── memory/             # Memory introspection (optional feature)
│
├── input/
│   ├── keyboard.rs     # Key event handling, security filtering
│   └── state.rs        # Input state, system command processor
│
├── output/
│   └── display.rs      # Display rendering
│
├── proxy/              # Reverse proxy (hyper-based)
│   ├── config.rs       # Proxy configuration types
│   ├── handler.rs      # HTTP/HTTPS proxy with configurable bind address
│   ├── manager.rs      # Route management
│   └── types.rs        # ProxyRoute, ProxyTarget
│
├── server/
│   ├── acme.rs         # Let's Encrypt ACME client (HTTP-01, auto-renewal)
│   ├── config.rs       # Server version and metadata
│   ├── manager.rs      # Server lifecycle management
│   ├── shared.rs       # Global singletons (context, registry, proxy manager, auto-start)
│   ├── types.rs        # ServerInfo, ServerStatus, ServerContext
│   ├── persistence.rs  # Server registry (servers.list) with fallback
│   ├── middleware.rs    # Middleware stack (API-Key auth, rate limiting, logging, detection)
│   ├── redirect.rs     # HTTP -> HTTPS redirect server
│   ├── tls.rs          # TLS certificate generation with production domain SANs
│   ├── watchdog.rs     # File watcher + WebSocket hot reload
│   ├── logging.rs      # Structured JSON logging with rotation
│   ├── handlers/web/   # actix-web request handlers
│   │   ├── api.rs      # REST API + File Upload API endpoints
│   │   ├── assets.rs   # Static assets (CSS, JS, fonts)
│   │   ├── logs.rs     # Log viewer and raw log API
│   │   ├── server.rs   # File serving with path traversal protection
│   │   └── templates.rs # Dashboard template
│   └── utils/
│       ├── port.rs     # Port availability checking
│       └── validation.rs # Server name validation
│
├── setup/
│   └── setup_toml.rs   # First-run TOML generation
│
└── ui/                 # TUI rendering (ratatui + crossterm)
    ├── color.rs        # Color parsing and AppColor
    ├── cursor.rs       # Cursor types and blinking
    ├── screen.rs       # Screen management
    ├── terminal.rs     # Terminal setup/teardown
    ├── viewport.rs     # Viewport calculations
    └── widget.rs       # Widget traits (Widget, CursorWidget, AnimatedWidget)
```

---

## Tech Stack

| Component        | Library                    |
|------------------|----------------------------|
| TUI              | ratatui + crossterm         |
| Web Server       | actix-web 4                 |
| Reverse Proxy    | hyper 0.14                  |
| Async Runtime    | tokio                       |
| TLS              | rustls + rcgen              |
| ACME/Let's Encrypt | ring + base64 + reqwest  |
| Cryptography     | ring (HMAC-SHA256)          |
| File Watching    | notify 6                    |
| WebSocket        | actix-web-actors            |
| Serialization    | serde + serde_json + toml   |
| Environment      | dotenvy                     |
| Logging          | log + env_logger + chrono   |
| Containerization | Docker (multi-stage build)  |

---

## Version History

- **v0.3.8** — First official public release (stable, Docker-ready, production-tested)
- **v0.1.0 – v0.3.7** — Internal development builds, developed and tested daily since February 2025

---

## License

### Dual-Licensing Model

1. **Community License (GPLv3)** — Free for private and non-commercial use
2. **Commercial License** — Required for commercial applications

Commercial licensing inquiries: [l.ersen@icloud.com](mailto:l.ersen@icloud.com)

---

## Contact

- **Email:** [l.ersen@icloud.com](mailto:l.ersen@icloud.com)
- **GitHub:** [LEVOGNE/rush.sync.server](https://github.com/LEVOGNE/rush.sync.server)
- **Issues:** [GitHub Issues](https://github.com/LEVOGNE/rush.sync.server/issues)
