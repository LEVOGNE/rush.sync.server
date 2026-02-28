# Rush Sync Server — Production Setup Guide

Your website online in 5 minutes. No nginx, no Docker knowledge, no reverse proxy setup required.

**Version:** 0.3.8 (first official public version)
**Development:** v0.1.0 – v0.3.7 were internal builds, developed and tested daily since February 2025.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [DNS Setup](#dns-setup)
5. [Starting the Server](#starting-the-server)
6. [Creating a Website](#creating-a-website)
7. [Deploying Files](#deploying-files)
8. [Accessing & Testing](#accessing--testing)
9. [Hosting Multiple Sites](#hosting-multiple-sites)
10. [Home Server (Fritz!Box)](#home-server-fritzbox)
11. [Checklist](#checklist)
12. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Option A: Docker (recommended)

- Docker + Docker Compose installed
- That's all — no Rust, no compiler needed

### Option B: Native (without Docker)

- A Linux server (VPS/root) or macOS with a public IP
- Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- A domain (e.g. `example.com`)

---

## Installation

### Option A: Docker (recommended)

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
docker compose up -d
```

This automatically starts:
- A default server on port 8000 (HTTP) and 9000 (HTTPS)
- The reverse proxy on port 3000 (HTTP) and 3443 (HTTPS)
- A Docker-optimized `rush.toml` with `bind_address = "0.0.0.0"`

Immediately accessible: `http://localhost:8000`

### Option B: From crates.io

```bash
cargo install rush-sync-server
```

### Option C: From Source

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
cargo build --release
cp target/release/rush-sync /usr/local/bin/
```

---

## Configuration

### Docker

With Docker, the configuration is generated **automatically** on the first start (`docker-entrypoint.sh`). You don't need to adjust anything manually.

To set the API key, edit `.env.docker`:

```env
RSS_API_KEY=your-secret-key
```

Then restart the container:

```bash
docker compose down && docker compose up -d
```

For advanced customizations, you can look inside the container:

```bash
# Open a shell in the container
docker compose exec rush-sync sh

# Show config
cat /app/.rss/rush.toml
```

### Native (without Docker)

On the first start, `rush.toml` is created automatically:

```bash
rush-sync        # First start generates the config
# Exit immediately (Ctrl+C)
```

Then edit `.rss/rush.toml`:

```toml
[server]
bind_address = "0.0.0.0"                  # Accessible from outside
production_domain = "example.com"          # Your real domain
api_key = "$hmac-sha256$..."               # Generate hash: rush-sync --hash-key your-secret-key

# Let's Encrypt (automatic HTTPS certificates)
use_lets_encrypt = true
acme_email = "admin@example.com"

[proxy]
enabled = true
port = 80                                  # Standard HTTP (required for Let's Encrypt)
bind_address = "0.0.0.0"
```

### Setting the API Key

There are three ways to configure the API key:

**1. Environment variable (recommended for Docker, CI/CD, systemd):**

```bash
# .env file (automatically loaded via dotenvy)
RSS_API_KEY=your-secret-key

# Or export directly
export RSS_API_KEY=your-secret-key
```

The env var overrides the TOML value and is **never** written back to the config file.

**2. HMAC-SHA256 hash in TOML (recommended — key is never stored in plain text):**

```bash
# Generate hash
rush-sync --hash-key your-secret-key
# Output: $hmac-sha256$aBcDeFgH...

# In Docker:
docker compose run --rm --entrypoint /app/rush-sync rush-sync --hash-key your-secret-key
```

```toml
[server]
api_key = "$hmac-sha256$aBcDeFgH..."
```

**3. Plain text in TOML (simplest option):**

```toml
[server]
api_key = "my-secret-key"
```

### Important Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `server.bind_address` | `127.0.0.1` | `0.0.0.0` = all interfaces, accessible from outside |
| `server.production_domain` | `localhost` | Your real domain for TLS and proxy routing |
| `server.api_key` | `""` | Protects `/api/*` and `/.rss/*` endpoints |
| `server.use_lets_encrypt` | `false` | Automatic Let's Encrypt certificates |
| `server.acme_email` | `""` | Email for Let's Encrypt notifications |
| `server.rate_limit_rps` | `100` | Max requests per second per IP |
| `proxy.port` | `3000` | Proxy HTTP port (`80` for production / Let's Encrypt) |
| `proxy.bind_address` | `127.0.0.1` | `0.0.0.0` for public proxy access |

---

## DNS Setup

Set two DNS records at your domain provider:

```
example.com      A    123.45.67.89
*.example.com    A    123.45.67.89
```

(`123.45.67.89` = IP of your server)

The wildcard entry (`*`) ensures that every subdomain automatically points to your server.

**Wait:** DNS changes can take up to 24 hours, but usually only a few minutes.

Check if it works:

```bash
dig +short example.com
dig +short myapp.example.com
# Both should show: 123.45.67.89
```

> **Important:** Do not use a CNAME for the wildcard! The wildcard must be an **A record** that points directly to the IP.

---

## Starting the Server

### Docker

```bash
# In the foreground (with log output)
docker compose up

# In the background
docker compose up -d

# Show logs
docker compose logs -f

# Stop
docker compose down

# Complete reset (delete config + data)
docker compose down -v
```

### Native — Headless Mode

```bash
# Headless mode (no terminal needed, ideal for servers)
rush-sync --headless
```

### Native — As a systemd Service

For automatic restart on crash or server reboot:

```bash
sudo tee /etc/systemd/system/rush-sync.service > /dev/null << 'EOF'
[Unit]
Description=Rush Sync Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/rush-sync
ExecStart=/usr/local/bin/rush-sync --headless
EnvironmentFile=-/opt/rush-sync/.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo mkdir -p /opt/rush-sync
sudo chown www-data:www-data /opt/rush-sync
sudo systemctl enable rush-sync
sudo systemctl start rush-sync
```

---

## Creating a Website

### Via TUI (interactive)

```bash
rush-sync
# Type in the terminal:
create myapp
start myapp
```

### Via API (headless / Docker)

```bash
# Check server status
curl -H "X-API-Key: your-key" http://localhost:8000/api/status

# Upload files (see next section)
```

> **Note:** With Docker, a default server on port 8000 is automatically created and started on the first launch.

---

## Deploying Files

### Option A: Via Sync Push (recommended for native installation)

Synchronize directly from your local machine to the server — fast, secure, with rsync:

```bash
# 1. Set up remote profile (one-time)
remote add prod deploy@example.com /opt/rush-sync/www/myapp-[8000]

# Optional: Specify SSH key
remote add prod deploy@example.com /opt/rush-sync/www/myapp-[8000] --port 22 --identity ~/.ssh/id_ed25519

# 2. Test connection
sync test prod

# 3. Upload files (rsync, only changes)
sync push prod ./www

# With --delete: Also delete files on the remote that no longer exist locally
sync push prod ./www --delete

# Preview what would happen (without actually making changes)
sync push prod ./www --dry-run

# Download files from the server
sync pull prod ./local-backup
```

Additional remote actions:

```bash
# Restart service on the server
sync restart prod rush-sync

# Execute any command on the server
sync exec prod "ls -la /opt/rush-sync"

# Git pull on the server
sync git-pull prod main
```

### Option B: Via File Upload API (ideal for Docker & CI/CD)

Upload files directly via `curl` — no SSH access needed:

```bash
# Upload a single file
curl -X PUT -H "X-API-Key: your-key" \
  --data-binary @index.html \
  http://myapp.example.com/api/files/index.html

# Upload CSS
curl -X PUT -H "X-API-Key: your-key" \
  --data-binary @style.css \
  http://myapp.example.com/api/files/style.css

# Upload to a subdirectory (created automatically)
curl -X PUT -H "X-API-Key: your-key" \
  --data-binary @logo.png \
  http://myapp.example.com/api/files/images/logo.png

# List files
curl -H "X-API-Key: your-key" \
  http://myapp.example.com/api/files

# Delete a file
curl -X DELETE -H "X-API-Key: your-key" \
  http://myapp.example.com/api/files/old-page.html
```

### Option C: Via SCP / Docker CP

```bash
# Via SCP (native)
scp -r ./my-website/* user@example.com:/opt/rush-sync/www/myapp-[8000]/

# Via Docker CP (copy into the container)
docker compose cp ./my-website/. rush-sync:/app/www/default-\[8000\]/
```

### File Structure

```
www/myapp-[8000]/
├── index.html      <-- Your start page
├── style.css
├── app.js
└── images/
    └── logo.png
```

Changes go **live immediately** — Hot Reload via WebSocket automatically refreshes the browser.

---

## Accessing & Testing

### Via the Reverse Proxy (Production)

```
http://myapp.example.com           # Via subdomain
http://default.example.com         # Docker default server
```

### Directly (without proxy)

```
http://localhost:8000               # Locally
http://example.com:8000             # From outside (if firewall port 8000 is open)
```

### Dashboard

```
http://localhost:8000/.rss/         # Web UI with status, metrics, API docs
```

### Testing the API

```bash
# Health check (always public, no key needed)
curl http://localhost:8000/api/health

# Status (protected when api_key is set)
curl -H "X-API-Key: your-key" http://localhost:8000/api/status

# Metrics
curl -H "X-API-Key: your-key" http://localhost:8000/api/metrics
```

---

## Hosting Multiple Sites

Each site automatically gets its own subdomain:

```bash
create blog
create api
create docs
start all
```

Result:

```
blog.example.com   ->  Your blog       (Port 8000)
api.example.com    ->  Your API        (Port 8001)
docs.example.com   ->  Your docs       (Port 8002)
```

### Deployment with Sync for Multiple Sites

```bash
# Set up remote profiles per site
remote add blog deploy@example.com /opt/rush-sync/www/blog-[8000]
remote add api  deploy@example.com /opt/rush-sync/www/api-[8001]
remote add docs deploy@example.com /opt/rush-sync/www/docs-[8002]

# Show all profiles
remote list

# Deploy each one
sync push blog ./blog-site
sync push api  ./api-build
sync push docs ./docs-output
```

The proxy routes automatically based on the subdomain — no nginx, no Apache, no reverse proxy config.

---

## Home Server (Fritz!Box)

Rush Sync Server also runs at home behind a router. Here's how to set it up:

### 1. Start Docker

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
docker compose up -d
```

### 2. Fritz!Box Port Forwarding

In the Fritz!Box menu under **Internet > Permits > Port Forwarding**:

| External | Internal (Docker Host) | Protocol |
|----------|----------------------|----------|
| Port 80 | 192.168.x.x:3000 | TCP |
| Port 443 | 192.168.x.x:3443 | TCP |

(`192.168.x.x` = IP of your machine on the local network)

> **Important:** Only use targeted port forwarding — **no** "Exposed Host"!

### 3. DNS at Your Domain Provider

```
example.com      A    <your-public-ip>
*.example.com    A    <your-public-ip>
```

You can find your public IP at: https://ifconfig.me or in the Fritz!Box menu under **Internet > Online Monitor**.

### 4. DynDNS (with a dynamic IP)

If your internet provider changes the IP regularly:

- Fritz!Box: Activate **Internet > Permits > DynDNS**
- Set up a DynDNS service (e.g. No-IP, DuckDNS, or directly at your domain provider)
- Then point the DNS A record to the DynDNS address as a CNAME

### 5. Testing

```bash
# From a different device (not from the same network!)
curl http://default.example.com

# Or check DNS
dig +short example.com
```

> **Note:** Some routers (including Fritz!Box) block access to your own public IP from the internal network. Test from outside (e.g. mobile data) or set the DNS server on your machine to `8.8.8.8`.

---

## Checklist

### Docker Deployment

- [ ] Docker + Docker Compose installed
- [ ] `RSS_API_KEY` set in `.env.docker`
- [ ] `docker compose up -d` is running
- [ ] DNS: `A` record and `*.` wildcard point to server IP
- [ ] Ports accessible (8000, 3000, 3443)

### Native Installation

- [ ] Server has a public IP
- [ ] `bind_address = "0.0.0.0"` in `[server]` and `[proxy]`
- [ ] `production_domain` = your real domain
- [ ] `api_key` set or `RSS_API_KEY` in `.env` (required for public access!)
- [ ] DNS: `A` record and `*.` wildcard point to server IP
- [ ] Firewall: Ports 80 and 443 are open
- [ ] `use_lets_encrypt = true` for automatic HTTPS certificates
- [ ] `rush-sync --headless` is running

### Home Server

- [ ] Docker is running on the local machine
- [ ] Fritz!Box port forwarding: 80 → 3000, 443 → 3443
- [ ] DNS A records point to public IP
- [ ] DynDNS configured (if dynamic IP)
- [ ] Test from external device successful

---

## Troubleshooting

### Site not reachable?

```bash
# Check DNS
dig +short myapp.example.com

# Docker container running?
docker compose ps
docker compose logs --tail 50

# Port open? (native installation)
sudo ufw allow 80
sudo ufw allow 443
# or
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Is the service running? (systemd)
systemctl status rush-sync
```

### API returns 401?

```bash
# Send the API key (always the plain text key, regardless of TOML hash or env var)
curl -H "X-API-Key: your-secret-key" http://localhost:8000/api/status

# Key stored as hash in TOML? Still send the plain text key in the header!
# rush-sync --hash-key your-secret-key  -> $hmac-sha256$... for rush.toml
# curl still gets the plain text key
```

### Docker container won't start?

```bash
# Show logs
docker compose logs -f

# Complete reset and restart
docker compose down -v
docker compose up

# Look inside the container
docker compose exec rush-sync sh
cat /app/.rss/rush.toml
ls -la /app/www/
```

### HTTPS certificate not working?

With `use_lets_encrypt = true`, certificates are automatically obtained from Let's Encrypt. Requirements:
- Port 80 must be accessible from outside (for HTTP-01 challenge)
- DNS must correctly point to the server
- `production_domain` must be set (not `localhost`)

Certificates are automatically checked every 24 hours and renewed 30 days before expiration.

If Let's Encrypt doesn't work, place manual certificates:

```
.rss/certs/example.com.fullchain.pem
.rss/certs/example.com.privkey.pem
```

### Fritz!Box: Access only works from outside?

Many routers (including Fritz!Box) do not support "NAT Loopback" — access to your own public IP from the internal network is blocked.

Solutions:
1. **Change the DNS server on your machine:** Set to `8.8.8.8` (Google) or `1.1.1.1` (Cloudflare) to bypass the Fritz!Box DNS cache
2. **Test from outside:** Use mobile data or a different network
3. **Access locally directly:** `http://localhost:8000` always works

### Hot Reload not working?

Hot Reload requires a WebSocket connection. Check:
- Check the browser console (F12) for WebSocket errors
- Files must be in the correct directory: `www/{name}-[{port}]/`
- Supported file types: HTML, CSS, JS, JSON, SVG, images
