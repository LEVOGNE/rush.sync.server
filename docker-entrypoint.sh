#!/bin/sh
set -e

CONFIG_DIR="/app/.rss"
CONFIG_FILE="$CONFIG_DIR/rush.toml"

# Generate Docker-optimized config if none exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo "[docker-entrypoint] No rush.toml found — generating Docker config..."
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_FILE" << 'TOML'
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
port_range_end = 8200
max_concurrent = 50
shutdown_timeout = 5
startup_delay_ms = 500
workers = 1
auto_open_browser = false
bind_address = "0.0.0.0"

enable_https = true
https_port_offset = 1000
cert_dir = ".rss/certs"
auto_cert = true
cert_validity_days = 365

use_lets_encrypt = true
production_domain = "rush-sync-server.com"
acme_email = "info@levogne.de"
api_key = ""
rate_limit_rps = 100
rate_limit_enabled = true

[proxy]
enabled = true
port = 3000
https_port_offset = 443
bind_address = "0.0.0.0"
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

[theme.light]
output_bg = "White"
output_text = "Black"
output_cursor = "PIPE"
output_cursor_color = "Black"
input_bg = "Black"
input_text = "White"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "White"

[theme.green]
output_bg = "Black"
output_text = "Green"
output_cursor = "BLOCK"
output_cursor_color = "Green"
input_bg = "LightGreen"
input_text = "Black"
input_cursor_prefix = "$ "
input_cursor = "BLOCK"
input_cursor_color = "Black"

[theme.blue]
output_bg = "White"
output_text = "LightBlue"
output_cursor = "UNDERSCORE"
output_cursor_color = "Blue"
input_bg = "Blue"
input_text = "White"
input_cursor_prefix = "> "
input_cursor = "UNDERSCORE"
input_cursor_color = "White"
TOML
    echo "[docker-entrypoint] Config created at $CONFIG_FILE"
else
    echo "[docker-entrypoint] Using existing rush.toml"
fi

# Seed a default server for auto-start if no registry exists
REGISTRY_FILE="$CONFIG_DIR/servers.list"
if [ ! -f "$REGISTRY_FILE" ]; then
    echo "[docker-entrypoint] Creating default server (port 8000, auto-start)..."
    TIMESTAMP=$(date +%s)
    CREATED_AT=$(date '+%Y-%m-%d %H:%M:%S')
    cat > "$REGISTRY_FILE" << JSON
[{"id":"docker-default-001","name":"default","port":8000,"status":"Stopped","created_at":"$CREATED_AT","created_timestamp":$TIMESTAMP,"auto_start":true,"last_started":null,"start_count":0}]
JSON
    # Create server web directory with landing page
    mkdir -p /app/www/default-\[8000\]
    cp /usr/local/share/landing.html /app/www/default-\[8000\]/index.html
    cp /usr/local/share/README.md /app/www/default-\[8000\]/README.md
    cp /usr/local/share/STARTUP.md /app/www/default-\[8000\]/STARTUP.md
    cp /usr/local/share/BENCHMARK.md /app/www/default-\[8000\]/BENCHMARK.md
    echo "[docker-entrypoint] Default server seeded"
fi

# Always sync downloads directory (binaries update on every rebuild)
WWW_DIR="/app/www/default-[8000]"
if [ -d "$WWW_DIR" ]; then
    mkdir -p "$WWW_DIR/downloads"
    cp /usr/local/share/downloads/* "$WWW_DIR/downloads/" 2>/dev/null || true
    echo "[docker-entrypoint] Downloads synced to $WWW_DIR/downloads/"
fi

# Pass all arguments to rush-sync (default: --headless)
exec /app/rush-sync "${@:---headless}"
