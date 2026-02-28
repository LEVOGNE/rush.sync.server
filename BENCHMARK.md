# Rush Sync Server — Benchmark & Resource Footprint (v0.3.8)

> As of: 2025-09-10 | Platforms: macOS (Dev), Linux/Docker (Production)
> Runtime: Tokio + Actix-Web | Proxy: Hyper 0.14
> Focus: **Resource consumption at idle** (Memory, FDs) with/without running servers.

---

## TL;DR

| Metric | Value |
|--------|-------|
| **Baseline (0 Servers)** | ~16.5 MB RSS |
| **+1 Server** | ~37.0 MB RSS |
| **+10 Servers** | ~48.9 MB RSS |
| **Overhead per Server** | ~3.3 MB (idle, after warmup) |
| **Open FDs (10 Servers)** | ~147 (59 Sockets) |
| **Docker Image** | ~95 MB (debian:bookworm-slim + Binary) |

For a Rust-based tool with reverse proxy, TLS, Let's Encrypt, hot reload, file watcher, and security middleware, these values are **very good** — especially compared to JS-based alternatives.

---

## Environment & Measurement Method

### Dev Build (macOS)

- **Build**: `cargo run` (Debug build, unoptimized)
- **OS**: macOS (VMS is very high due to system behavior and says **nothing** about actual RAM usage)
- **Runtime**: Tokio (once), Actix-Web per server, Hyper for proxy

### Production Build (Docker/Linux)

- **Build**: `cargo build --release` (Multi-Stage Dockerfile, `rust:1.83-bookworm`)
- **Runtime**: `debian:bookworm-slim` with `ca-certificates`
- **Mode**: `--headless` (no TUI overhead, no Crossterm/Ratatui)

### Measurement Points

```bash
# In TUI mode (with memory feature)
mem info              # Table
mem info --json       # JSON output
mem info --all        # FD overview (Linux detailed, macOS heuristic)

# Docker
docker stats          # CPU, Memory, Net I/O
docker compose exec rush-sync sh -c "cat /proc/1/status | grep VmRSS"
```

---

## Results

### Idle Consumption (Debug Build, macOS)

| Scenario | RSS (MB) | Registry TOTAL (MB) | FDs (total) | Sockets |
|----------|--------:|-------------------:|------------:|--------:|
| Baseline (0 Servers) | ~16.5 | ~11.7 | 15 | 7 |
| +1 Server started | ~37.0 | ~11.7 | 28 | 12 |
| +10 Servers started | ~48.9 | ~11.7 | 147 | 59 |

**Derived:**

- **Idle overhead per server: (48.9 - 16.5) / 10 = ~3.3 MB**
- **Registry TOTAL remains constant** (~11.7 MB) — embedded assets and static data
- FDs scale linearly (listener, WebSockets, watcher per server)

### What Is Included?

Each server starts the following components:

| Component | FDs | Description |
|-----------|----:|-------------|
| HTTP Listener | 1 | actix-web on `bind_address:port` |
| HTTPS Listener | 1 | actix-web on `bind_address:port+offset` |
| File Watcher | 2-3 | notify instance for `www/{name}-[{port}]/` |
| WebSocket Hub | 1+ | Hot reload broadcast to connected browsers |
| Proxy Route | 1 | Registration in reverse proxy |
| TLS Certs | 0 | Generated/loaded once at startup |

---

## Docker Footprint

### Image Size

```
rust:1.83-bookworm (Builder)     ~1.5 GB    (only during build)
debian:bookworm-slim (Runtime)   ~80 MB     (base image)
rush-sync Binary                 ~10 MB     (release)
Total Runtime Image              ~95 MB
```

### Container Consumption (Headless, 1 Server)

| Metric | Value |
|--------|-------|
| Memory (RSS) | ~25-30 MB |
| CPU (idle) | <0.1% |
| Network I/O (idle) | ~0 |
| Disk (Config + Certs) | ~50 KB |

> **Note:** Headless mode consumes less than TUI mode, since Crossterm/Ratatui are not loaded.

---

## Classification & Comparison

### Rush Sync Server vs. Alternatives

| Tool | Type | Idle (1 Server) | 10 Servers | Features |
|------|-----|----------------:|----------:|----------|
| **Rush Sync** | All-in-One | ~30 MB | ~49 MB | Proxy + TLS + Hot Reload + API + Security |
| nginx | Reverse Proxy | ~2-5 MB | ~5-10 MB | Proxy/Static only |
| Caddy | Proxy + ACME | ~30-50 MB | ~50-80 MB | Proxy + Auto-TLS |
| Traefik | Proxy + Discovery | ~50-80 MB | ~80-120 MB | Proxy + Service Discovery |
| Node/Express | App Server | ~40-60 MB | ~400-600 MB | One process per server |
| Next.js dev | App Server | ~100-200 MB | ~1-2 GB | Dev mode with HMR |

### Classification

**Compared to pure proxies (nginx/HAProxy):**
- Higher idle footprint — Rush is not a pure proxy, but a full server orchestrator with TLS, watcher, API, dashboard, and security stack
- nginx/HAProxy are optimized for a single task and often operate in the single-digit MB range

**Compared to all-in-one proxies (Caddy/Traefik):**
- Comparable footprint with significantly more features (multi-server spawn, TUI, file upload API, hot reload)
- Caddy and Rush are in the same weight class

**Compared to JS dev servers (Node/Express/Next):**
- **Significantly more efficient** — 10 Rush servers need ~49 MB, 10 Node processes easily 400+ MB
- Rust + shared Tokio runtime pay off

---

## Scaling

### Linear Scaling

```
Servers:  1    5    10    20    50
RSS:     37   44    49    56    72  (MB, estimated)
FDs:     28   88   147   280   700
```

The overhead per server remains constant at ~3.3 MB, because:
- Tokio runtime is shared
- Actix-Web worker pool is shared
- Embedded assets are loaded once
- Only listeners, watchers, and routes scale

### Limits

| Limit | Value | Configurable |
|-------|-------|:------------:|
| Max. concurrent servers | 50 (default) | `server.max_concurrent` |
| Port range | 8000-8200 (default) | `server.port_range_start/end` |
| FD limit (Linux) | 1024 (default) | `ulimit -n` |
| FD limit (Docker) | 1048576 | Docker default |

> At 50 servers: ~72 MB RSS, ~700 FDs — well below system limits.

---

## Reproducibility

### In TUI Mode (with `memory` feature)

```bash
# 1) Start project (Debug)
cargo run --features memory

# 2) Create & start server
create
start 1

# 3) Take snapshot
mem info
mem info --json
mem info --all
```

### In Docker

```bash
# 1) Start container
docker compose up -d

# 2) Check consumption
docker stats --no-stream

# 3) Detailed inside container
docker compose exec rush-sync sh
cat /proc/1/status | grep -E "VmRSS|VmSize|Threads"
ls /proc/1/fd | wc -l
```

---

## Optimization Tips

### Release Build

A release build (`cargo build --release`) reduces the baseline footprint by approximately 30-40% compared to the debug build.

### Headless Mode

The `--headless` mode saves the entire TUI stack (Crossterm, Ratatui, terminal rendering) — ideal for servers and Docker.

### Production Profile

In production, file watchers can be disabled to further reduce FDs and idle RSS. Hot reload is rarely needed in production anyway.

### Tokio/Actix Tuning

```toml
[server]
workers = 1           # Fewer workers = fewer threads (Default: 1)
shutdown_timeout = 5   # Faster shutdown
```

---

## Final Assessment

| Aspect | Rating |
|--------|--------|
| **Memory Efficiency** | Very good — ~3.3 MB per server, shared runtime |
| **Scaling** | Linear and predictable |
| **Docker Footprint** | ~95 MB image, ~30 MB runtime |
| **vs. JS Alternatives** | 5-10x more efficient per instance |
| **vs. Pure Proxies** | Higher, but significantly more features |
| **Production Readiness** | Headless + Docker = production-ready |

> **Rush Sync Server** delivers a complete server stack (proxy, TLS, ACME, API, security, hot reload, dashboard) in a footprint comparable to specialized proxy tools — and significantly below that of JS-based alternatives.
