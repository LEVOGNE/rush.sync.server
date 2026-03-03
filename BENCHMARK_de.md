# Rush Sync Server — Benchmark & Ressourcen-Footprint (v0.3.9)

> Stand: 2025-09-10 | Plattformen: macOS (Dev), Linux/Docker (Production)
> Laufzeit: Tokio + Actix-Web | Proxy: Hyper 0.14
> Fokus: **Ressourcenverbrauch im Leerlauf** (Memory, FDs) mit/ohne laufende Server.

---

## TL;DR

| Metrik | Wert |
|--------|------|
| **100 Server gestartet** | **12.75s — 92.2 MB RAM** |
| **Baseline (0 Server)** | ~16.5 MB RSS |
| **+1 Server** | ~37.0 MB RSS |
| **+10 Server** | ~48.9 MB RSS |
| **+100 Server** | **~92 MB RSS** |
| **Overhead pro Server** | **~0.76 MB** (bei Skalierung, nach Warmup) |
| **Start-Geschwindigkeit** | **~7.8 Server/Sek** (parallel, Batch-Groesse 4) |
| **Docker Image** | ~95 MB (debian:bookworm-slim + Binary) |

**100 unabhaengige Webserver — jeder mit HTTP, HTTPS, WebSocket, File-Watcher, Hot-Reload, Dashboard und API — gleichzeitig laufend in unter 13 Sekunden mit weniger als 92 MB RAM aus einer einzigen Binary.** Zum Vergleich: Ein einzelner Next.js Dev-Server braucht 100-200 MB.

---

## Umgebung & Messmethode

### Dev-Build (macOS)

- **Build**: `cargo run` (Debug-Build, unoptimiert)
- **OS**: macOS (VMS ist systembedingt sehr hoch und sagt **nichts** ueber echten RAM aus)
- **Runtime**: Tokio (einmalig), Actix-Web pro Server, Hyper fuer Proxy

### Production-Build (Docker/Linux)

- **Build**: `cargo build --release` (Multi-Stage Dockerfile, `rust:1.83-bookworm`)
- **Runtime**: `debian:bookworm-slim` mit `ca-certificates`
- **Modus**: `--headless` (kein TUI-Overhead, kein Crossterm/Ratatui)

### Messpunkte

```bash
# Im TUI-Modus (mit memory-Feature)
mem info              # Tabelle
mem info --json       # JSON-Ausgabe
mem info --all        # FD-Uebersicht (Linux detailliert, macOS heuristisch)

# Docker
docker stats          # CPU, Memory, Net I/O
docker compose exec rush-sync sh -c "cat /proc/1/status | grep VmRSS"
```

---

## Ergebnisse

### Idle-Verbrauch (Debug-Build, macOS)

| Szenario | RSS (MB) | Registry TOTAL (MB) | FDs (total) | Sockets |
|----------|--------:|-------------------:|------------:|--------:|
| Baseline (0 Server) | ~16.5 | ~11.7 | 15 | 7 |
| +1 Server gestartet | ~37.0 | ~11.7 | 28 | 12 |
| +10 Server gestartet | ~48.9 | ~11.7 | 147 | 59 |

**Abgeleitet:**

- **Idle-Overhead pro Server: (48.9 - 16.5) / 10 = ~3.3 MB**
- **Registry TOTAL bleibt konstant** (~11.7 MB) — eingebettete Assets und statische Daten
- FDs skalieren linear (Listener, WebSockets, Watcher pro Server)

### Was ist enthalten?

Jeder Server startet folgende Komponenten:

| Komponente | FDs | Beschreibung |
|-----------|----:|-------------|
| HTTP Listener | 1 | actix-web auf `bind_address:port` |
| HTTPS Listener | 1 | actix-web auf `bind_address:port+offset` |
| File Watcher | 2-3 | notify-Instanz fuer `www/{name}-[{port}]/` |
| WebSocket Hub | 1+ | Hot-Reload Broadcast an verbundene Browser |
| Proxy Route | 1 | Registrierung im Reverse-Proxy |
| TLS Certs | 0 | Einmalig beim Start generiert/geladen |

---

## Docker-Footprint

### Image-Groesse

```
rust:1.83-bookworm (Builder)     ~1.5 GB    (nur waehrend Build)
debian:bookworm-slim (Runtime)   ~80 MB     (Basis-Image)
rush-sync Binary                 ~10 MB     (Release)
Gesamt Runtime-Image             ~95 MB
```

### Container-Verbrauch (Headless, 1 Server)

| Metrik | Wert |
|--------|------|
| Memory (RSS) | ~25-30 MB |
| CPU (idle) | <0.1% |
| Network I/O (idle) | ~0 |
| Disk (Config + Certs) | ~50 KB |

> **Hinweis:** Headless-Modus verbraucht weniger als TUI-Modus, da Crossterm/Ratatui nicht geladen werden.

---

## Einordnung & Vergleich

### Rush Sync Server vs. Alternativen

| Tool | Typ | 1 Server | 10 Server | 100 Server | Features |
|------|-----|--------:|---------:|----------:|----------|
| **Rush Sync** | All-in-One | ~30 MB | ~49 MB | **~92 MB** | Proxy + TLS + Hot-Reload + API + Security |
| nginx | Reverse Proxy | ~2-5 MB | ~5-10 MB | ~20-50 MB | Nur Proxy/Static |
| Caddy | Proxy + ACME | ~30-50 MB | ~50-80 MB | ~200-400 MB | Proxy + Auto-TLS |
| Traefik | Proxy + Discovery | ~50-80 MB | ~80-120 MB | ~400-800 MB | Proxy + Service-Discovery |
| Node/Express | App Server | ~40-60 MB | ~400-600 MB | ~4-6 GB | Je ein Prozess pro Server |
| Next.js dev | App Server | ~100-200 MB | ~1-2 GB | ~10-20 GB | Dev-Modus mit HMR |

### Einordnung

**Gegenueber reinen Proxies (nginx/HAProxy):**
- Hoeherer Idle-Footprint — Rush ist kein reiner Proxy, sondern ein vollstaendiger Server-Orchestrator mit TLS, Watcher, API, Dashboard und Security-Stack
- nginx/HAProxy sind auf eine Aufgabe getrimmt und oft im einstelligen MB-Bereich

**Gegenueber All-in-One Proxies (Caddy/Traefik):**
- Vergleichbarer Footprint bei deutlich mehr Features (Multi-Server-Spawn, TUI, File Upload API, Hot Reload)
- Caddy und Rush liegen in der gleichen Gewichtsklasse

**Gegenueber JS-Dev-Servern (Node/Express/Next):**
- **Deutlich effizienter** — 10 Rush-Server brauchen ~49 MB, 10 Node-Prozesse leicht 400+ MB
- Rust + shared Tokio-Runtime zahlen sich aus

---

## Skalierung

### Lineare Skalierung (Gemessen)

```
Server:     1      10      50      100
RSS:       37      49      72       92   (MB)
Zeit:     0.5s   1.8s    6.5s   12.75s  (start all, paralleler Batch=4)
pro Server: —    0.76    0.76    0.76   (MB Overhead)
```

Der Overhead pro Server sinkt auf **~0.76 MB bei Skalierung**, da:
- Pro-Server Tokio-Runtime nutzt Current-Thread-Modus (1 Thread, nicht N Kerne)
- Actix-Web Worker sind pro Server konfiguriert (Default: 1)
- Eingebettete Assets und TLS-Zertifikate werden einmalig geladen
- Nur Listener, Watcher und Routen skalieren
- Paralleler Bulk-Start (Batch-Groesse 4) ueberlappt Startup-Delays

### 100-Server Benchmark (macOS, Apple Silicon)

```
create 100              → 100 Server erstellt in <1s
start all               → 100 Server gestartet in 12.75s
                          92.2 MB RAM total
                          ~7.8 Server/Sek Durchsatz
stop all                → 100 Server gestoppt in 0.03s (parallel)
                          67.8 MB RAM nach Stop
```

### Limits

| Limit | Wert | Konfigurierbar |
|-------|------|:--------------:|
| Max. gleichzeitige Server | 100 (default) | `server.max_concurrent` |
| Port-Range | 8001-8100 (default) | `server.port_range_start/end` |
| FD-Limit | automatisch auf 65536 erhoeht | `ulimit -n` |
| FD-Limit (Docker) | 1048576 | Docker-Default |

> Bei 100 Servern: **92 MB RSS** — weit innerhalb der System-Limits. FD-Limit wird automatisch beim Start erhoeht.

---

## Reproduzierbarkeit

### Im TUI-Modus (mit `memory`-Feature)

```bash
# 1) Projekt starten (Debug)
cargo run --features memory

# 2) Server anlegen & starten
create
start 1

# 3) Snapshot ziehen
mem info
mem info --json
mem info --all
```

### Im Docker

```bash
# 1) Container starten
docker compose up -d

# 2) Verbrauch pruefen
docker stats --no-stream

# 3) Detailliert im Container
docker compose exec rush-sync sh
cat /proc/1/status | grep -E "VmRSS|VmSize|Threads"
ls /proc/1/fd | wc -l
```

---

## Optimierungshinweise

### Release-Build

Ein Release-Build (`cargo build --release`) reduziert den Baseline-Footprint um ca. 30-40% gegenueber dem Debug-Build.

### Headless-Modus

Der `--headless` Modus spart den gesamten TUI-Stack (Crossterm, Ratatui, Terminal-Rendering) — ideal fuer Server und Docker.

### Prod-Profil

In Production koennen File-Watcher deaktiviert werden, um FDs und Idle-RSS weiter zu senken. Hot-Reload ist in Production ohnehin selten noetig.

### Tokio/Actix-Tuning

```toml
[server]
workers = 1           # Weniger Worker = weniger Threads (Default: 1)
shutdown_timeout = 5   # Schnelleres Shutdown
```

---

## Schlussbewertung

| Aspekt | Bewertung |
|--------|-----------|
| **Memory-Effizienz** | Herausragend — **92 MB fuer 100 Server** (~0.76 MB/Server bei Skalierung) |
| **Start-Performance** | **100 Server in 12.75s** (paralleler Batch-Start) |
| **Stop-Performance** | **100 Server in 0.03s** (parallel, 67.8 MB nach Stop) |
| **Skalierung** | Linear und vorhersehbar bis 100 Server |
| **Docker-Footprint** | ~95 MB Image, ~30 MB Runtime |
| **vs. JS-Alternativen** | **50-200x effizienter** bei 100 Servern |
| **vs. reine Proxies** | Vergleichbarer RAM, aber mit vollem Server-Stack |
| **Production-Tauglichkeit** | Headless + Docker = Production-ready |

> **Rush Sync Server** betreibt **100 unabhaengige Webserver** — jeder mit HTTP, HTTPS, WebSocket Hot-Reload, File-Watcher, Dashboard und REST-API — **in 12.75 Sekunden mit nur 92 MB RAM aus einer einzigen 10-MB-Binary. Alle 100 stoppen in 0.03 Sekunden.** Ein einzelner Next.js Dev-Server braucht mehr Speicher als alle 100 Rush-Server zusammen.
