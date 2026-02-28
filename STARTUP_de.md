# Rush Sync Server — Production Setup Guide

Deine Website in 5 Minuten online. Kein nginx, kein Docker-Wissen, kein Reverse-Proxy-Setup.

**Version:** 0.3.8 (erste offizielle oeffentliche Version)
**Entwicklung:** v0.1.0 – v0.3.7 waren interne Builds, seit Februar 2025 taeglich entwickelt und getestet.

---

## Inhaltsverzeichnis

1. [Voraussetzungen](#voraussetzungen)
2. [Installation](#installation)
3. [Konfiguration](#konfiguration)
4. [DNS einrichten](#dns-einrichten)
5. [Server starten](#server-starten)
6. [Website erstellen](#website-erstellen)
7. [Dateien deployen](#dateien-deployen)
8. [Aufrufen & testen](#aufrufen--testen)
9. [Mehrere Seiten hosten](#mehrere-seiten-hosten)
10. [Home-Server (Fritz!Box)](#home-server-fritzbox)
11. [Checkliste](#checkliste)
12. [Troubleshooting](#troubleshooting)

---

## Voraussetzungen

### Option A: Docker (empfohlen)

- Docker + Docker Compose installiert
- Das ist alles — kein Rust, kein Compiler noetig

### Option B: Nativ (ohne Docker)

- Ein Linux-Server (VPS/Root) oder macOS mit oeffentlicher IP
- Rust installiert (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Eine Domain (z.B. `example.com`)

---

## Installation

### Option A: Docker (empfohlen)

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
docker compose up -d
```

Das startet automatisch:
- Einen Default-Server auf Port 8000 (HTTP) und 9000 (HTTPS)
- Den Reverse Proxy auf Port 3000 (HTTP) und 3443 (HTTPS)
- Eine Docker-optimierte `rush.toml` mit `bind_address = "0.0.0.0"`

Sofort erreichbar: `http://localhost:8000`

### Option B: Von crates.io

```bash
cargo install rush-sync-server
```

### Option C: Aus dem Source

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
cargo build --release
cp target/release/rush-sync /usr/local/bin/
```

---

## Konfiguration

### Docker

Bei Docker wird die Konfiguration **automatisch** beim ersten Start erzeugt (`docker-entrypoint.sh`). Du musst nichts manuell anpassen.

Um den API-Key zu setzen, bearbeite `.env.docker`:

```env
RSS_API_KEY=dein-geheimer-schluessel
```

Dann Container neu starten:

```bash
docker compose down && docker compose up -d
```

Fuer erweiterte Anpassungen kannst du in den Container schauen:

```bash
# Shell im Container oeffnen
docker compose exec rush-sync sh

# Config anzeigen
cat /app/.rss/rush.toml
```

### Nativ (ohne Docker)

Beim ersten Start wird `rush.toml` automatisch erstellt:

```bash
rush-sync        # Erster Start erzeugt die Config
# Sofort beenden (Ctrl+C)
```

Dann `.rss/rush.toml` bearbeiten:

```toml
[server]
bind_address = "0.0.0.0"                  # Von aussen erreichbar
production_domain = "example.com"          # Deine echte Domain
api_key = "$hmac-sha256$..."               # Hash erzeugen: rush-sync --hash-key dein-geheimer-schluessel

# Let's Encrypt (automatische HTTPS-Zertifikate)
use_lets_encrypt = true
acme_email = "admin@example.com"

[proxy]
enabled = true
port = 80                                  # Standard HTTP (noetig fuer Let's Encrypt)
bind_address = "0.0.0.0"
```

### API-Key setzen

Es gibt drei Wege, den API-Key zu konfigurieren:

**1. Umgebungsvariable (empfohlen fuer Docker, CI/CD, systemd):**

```bash
# .env Datei (wird automatisch via dotenvy geladen)
RSS_API_KEY=dein-geheimer-schluessel

# Oder direkt exportieren
export RSS_API_KEY=dein-geheimer-schluessel
```

Der Env-Var ueberschreibt den TOML-Wert und wird **nie** in die Config-Datei zurueckgeschrieben.

**2. HMAC-SHA256 Hash in TOML (empfohlen — Key nie im Klartext gespeichert):**

```bash
# Hash erzeugen
rush-sync --hash-key dein-geheimer-schluessel
# Ausgabe: $hmac-sha256$aBcDeFgH...

# In Docker:
docker compose run --rm --entrypoint /app/rush-sync rush-sync --hash-key dein-geheimer-schluessel
```

```toml
[server]
api_key = "$hmac-sha256$aBcDeFgH..."
```

**3. Klartext in TOML (einfachste Variante):**

```toml
[server]
api_key = "mein-geheimer-schluessel"
```

### Wichtige Einstellungen

| Einstellung | Default | Beschreibung |
|-------------|---------|--------------|
| `server.bind_address` | `127.0.0.1` | `0.0.0.0` = alle Interfaces, von aussen erreichbar |
| `server.production_domain` | `localhost` | Deine echte Domain fuer TLS und Proxy-Routing |
| `server.api_key` | `""` | Schuetzt `/api/*` und `/.rss/*` Endpoints |
| `server.use_lets_encrypt` | `false` | Automatische Let's Encrypt Zertifikate |
| `server.acme_email` | `""` | E-Mail fuer Let's Encrypt Benachrichtigungen |
| `server.rate_limit_rps` | `100` | Max Requests pro Sekunde pro IP |
| `proxy.port` | `3000` | Proxy HTTP Port (`80` fuer Production / Let's Encrypt) |
| `proxy.bind_address` | `127.0.0.1` | `0.0.0.0` fuer oeffentlichen Proxy-Zugang |

---

## DNS einrichten

Bei deinem Domain-Provider zwei DNS-Eintraege setzen:

```
example.com      A    123.45.67.89
*.example.com    A    123.45.67.89
```

(`123.45.67.89` = IP deines Servers)

Der Wildcard-Eintrag (`*`) sorgt dafuer, dass jede Subdomain automatisch auf deinen Server zeigt.

**Warten:** DNS-Aenderungen brauchen bis zu 24 Stunden, meistens aber nur wenige Minuten.

Pruefen ob es funktioniert:

```bash
dig +short example.com
dig +short myapp.example.com
# Beide sollten: 123.45.67.89
```

> **Wichtig:** Kein CNAME fuer Wildcard verwenden! Der Wildcard muss ein **A-Record** sein, der direkt auf die IP zeigt.

---

## Server starten

### Docker

```bash
# Im Vordergrund (mit Log-Ausgabe)
docker compose up

# Im Hintergrund
docker compose up -d

# Logs anzeigen
docker compose logs -f

# Stoppen
docker compose down

# Komplett zuruecksetzen (Config + Daten loeschen)
docker compose down -v
```

### Nativ — Headless-Modus

```bash
# Headless-Modus (kein Terminal noetig, ideal fuer Server)
rush-sync --headless
```

### Nativ — Als systemd-Service

Fuer automatischen Neustart bei Absturz oder Server-Reboot:

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

## Website erstellen

### Per TUI (interaktiv)

```bash
rush-sync
# Im Terminal eingeben:
create myapp
start myapp
```

### Per API (headless / Docker)

```bash
# Server-Status pruefen
curl -H "X-API-Key: dein-schluessel" http://localhost:8000/api/status

# Dateien hochladen (siehe naechster Abschnitt)
```

> **Hinweis:** Bei Docker wird beim ersten Start automatisch ein Default-Server auf Port 8000 erstellt und gestartet.

---

## Dateien deployen

### Option A: Per Sync Push (empfohlen fuer native Installation)

Vom lokalen Rechner direkt auf den Server synchronisieren — schnell, sicher, mit rsync:

```bash
# 1. Remote-Profil einrichten (einmalig)
remote add prod deploy@example.com /opt/rush-sync/www/myapp-[8000]

# Optional: SSH-Key angeben
remote add prod deploy@example.com /opt/rush-sync/www/myapp-[8000] --port 22 --identity ~/.ssh/id_ed25519

# 2. Verbindung testen
sync test prod

# 3. Dateien hochladen (rsync, nur Aenderungen)
sync push prod ./www

# Mit --delete: Dateien die lokal nicht mehr existieren auch remote loeschen
sync push prod ./www --delete

# Vorschau was passieren wuerde (ohne tatsaechlich zu aendern)
sync push prod ./www --dry-run

# Dateien vom Server herunterladen
sync pull prod ./local-backup
```

Weitere Remote-Aktionen:

```bash
# Service auf dem Server neustarten
sync restart prod rush-sync

# Beliebigen Befehl auf dem Server ausfuehren
sync exec prod "ls -la /opt/rush-sync"

# Git pull auf dem Server
sync git-pull prod main
```

### Option B: Per File Upload API (ideal fuer Docker & CI/CD)

Dateien direkt per `curl` hochladen — kein SSH-Zugang noetig:

```bash
# Einzelne Datei hochladen
curl -X PUT -H "X-API-Key: dein-schluessel" \
  --data-binary @index.html \
  http://myapp.example.com/api/files/index.html

# CSS hochladen
curl -X PUT -H "X-API-Key: dein-schluessel" \
  --data-binary @style.css \
  http://myapp.example.com/api/files/style.css

# In Unterordner hochladen (wird automatisch erstellt)
curl -X PUT -H "X-API-Key: dein-schluessel" \
  --data-binary @logo.png \
  http://myapp.example.com/api/files/images/logo.png

# Dateien auflisten
curl -H "X-API-Key: dein-schluessel" \
  http://myapp.example.com/api/files

# Datei loeschen
curl -X DELETE -H "X-API-Key: dein-schluessel" \
  http://myapp.example.com/api/files/alte-seite.html
```

### Option C: Per SCP / Docker CP

```bash
# Per SCP (nativ)
scp -r ./meine-website/* user@example.com:/opt/rush-sync/www/myapp-[8000]/

# Per Docker CP (in den Container kopieren)
docker compose cp ./meine-website/. rush-sync:/app/www/default-\[8000\]/
```

### Dateistruktur

```
www/myapp-[8000]/
├── index.html      <-- Deine Startseite
├── style.css
├── app.js
└── images/
    └── logo.png
```

Aenderungen werden **sofort live** — Hot Reload per WebSocket aktualisiert den Browser automatisch.

---

## Aufrufen & testen

### Ueber den Reverse Proxy (Production)

```
http://myapp.example.com           # Per Subdomain
http://default.example.com         # Docker Default-Server
```

### Direkt (ohne Proxy)

```
http://localhost:8000               # Lokal
http://example.com:8000             # Von aussen (wenn Firewall Port 8000 offen)
```

### Dashboard

```
http://localhost:8000/.rss/         # Web UI mit Status, Metriken, API-Docs
```

### API testen

```bash
# Health Check (immer oeffentlich, kein Key noetig)
curl http://localhost:8000/api/health

# Status (geschuetzt wenn api_key gesetzt)
curl -H "X-API-Key: dein-schluessel" http://localhost:8000/api/status

# Metriken
curl -H "X-API-Key: dein-schluessel" http://localhost:8000/api/metrics
```

---

## Mehrere Seiten hosten

Jede Seite bekommt automatisch eine eigene Subdomain:

```bash
create blog
create api
create docs
start all
```

Ergebnis:

```
blog.example.com   ->  Dein Blog       (Port 8000)
api.example.com    ->  Deine API       (Port 8001)
docs.example.com   ->  Deine Docs      (Port 8002)
```

### Deployment mit Sync fuer mehrere Seiten

```bash
# Remote-Profile pro Seite einrichten
remote add blog deploy@example.com /opt/rush-sync/www/blog-[8000]
remote add api  deploy@example.com /opt/rush-sync/www/api-[8001]
remote add docs deploy@example.com /opt/rush-sync/www/docs-[8002]

# Alle Profile anzeigen
remote list

# Jeweils deployen
sync push blog ./blog-site
sync push api  ./api-build
sync push docs ./docs-output
```

Der Proxy routet automatisch anhand der Subdomain — kein nginx, kein Apache, keine Reverse-Proxy-Config.

---

## Home-Server (Fritz!Box)

Rush Sync Server laeuft auch zuhause hinter einem Router. So richtest du es ein:

### 1. Docker starten

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
docker compose up -d
```

### 2. Fritz!Box Port-Weiterleitung

Im Fritz!Box-Menue unter **Internet > Freigaben > Portfreigaben**:

| Extern | Intern (Docker-Host) | Protokoll |
|--------|---------------------|-----------|
| Port 80 | 192.168.x.x:3000 | TCP |
| Port 443 | 192.168.x.x:3443 | TCP |

(`192.168.x.x` = IP deines Rechners im lokalen Netzwerk)

> **Wichtig:** Nur gezielte Port-Weiterleitungen verwenden — **kein** "Exposed Host"!

### 3. DNS bei deinem Domain-Anbieter

```
example.com      A    <deine-oeffentliche-IP>
*.example.com    A    <deine-oeffentliche-IP>
```

Deine oeffentliche IP findest du unter: https://ifconfig.me oder im Fritz!Box-Menue unter **Internet > Online-Monitor**.

### 4. DynDNS (bei dynamischer IP)

Falls dein Internet-Provider die IP regelmaessig wechselt:

- Fritz!Box: **Internet > Freigaben > DynDNS** aktivieren
- Einen DynDNS-Dienst einrichten (z.B. No-IP, DuckDNS, oder direkt bei deinem Domain-Anbieter)
- Dann den DNS-A-Record auf die DynDNS-Adresse als CNAME zeigen lassen

### 5. Testen

```bash
# Von einem anderen Geraet (nicht aus dem gleichen Netzwerk!)
curl http://default.example.com

# Oder DNS pruefen
dig +short example.com
```

> **Hinweis:** Manche Router (inkl. Fritz!Box) blockieren den Zugriff auf die eigene oeffentliche IP aus dem internen Netzwerk. Teste von extern (z.B. Mobilfunk) oder setze den DNS-Server am Rechner auf `8.8.8.8`.

---

## Checkliste

### Docker-Deployment

- [ ] Docker + Docker Compose installiert
- [ ] `RSS_API_KEY` in `.env.docker` gesetzt
- [ ] `docker compose up -d` laeuft
- [ ] DNS: `A`-Record und `*.`-Wildcard zeigen auf Server-IP
- [ ] Ports erreichbar (8000, 3000, 3443)

### Native Installation

- [ ] Server hat oeffentliche IP
- [ ] `bind_address = "0.0.0.0"` in `[server]` und `[proxy]`
- [ ] `production_domain` = deine echte Domain
- [ ] `api_key` gesetzt oder `RSS_API_KEY` in `.env` (Pflicht bei oeffentlichem Zugang!)
- [ ] DNS: `A`-Record und `*.`-Wildcard zeigen auf Server-IP
- [ ] Firewall: Port 80 und 443 sind offen
- [ ] `use_lets_encrypt = true` fuer automatische HTTPS-Zertifikate
- [ ] `rush-sync --headless` laeuft

### Home-Server

- [ ] Docker laeuft auf dem lokalen Rechner
- [ ] Fritz!Box Port-Weiterleitung: 80 → 3000, 443 → 3443
- [ ] DNS-A-Records zeigen auf oeffentliche IP
- [ ] DynDNS eingerichtet (falls dynamische IP)
- [ ] Test von externem Geraet erfolgreich

---

## Troubleshooting

### Seite nicht erreichbar?

```bash
# DNS pruefen
dig +short myapp.example.com

# Docker-Container laeuft?
docker compose ps
docker compose logs --tail 50

# Port offen? (native Installation)
sudo ufw allow 80
sudo ufw allow 443
# oder
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT
sudo iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Laeuft der Service? (systemd)
systemctl status rush-sync
```

### API gibt 401 zurueck?

```bash
# API-Key mitschicken (immer den Klartext-Key, egal ob TOML Hash oder Env-Var)
curl -H "X-API-Key: dein-geheimer-schluessel" http://localhost:8000/api/status

# Key als Hash in TOML? Im Header trotzdem den Klartext-Key senden!
# rush-sync --hash-key dein-geheimer-schluessel  -> $hmac-sha256$... fuer rush.toml
# curl bekommt weiterhin den Klartext-Key
```

### Docker-Container startet nicht?

```bash
# Logs anzeigen
docker compose logs -f

# Komplett zuruecksetzen und neu starten
docker compose down -v
docker compose up

# In den Container schauen
docker compose exec rush-sync sh
cat /app/.rss/rush.toml
ls -la /app/www/
```

### HTTPS-Zertifikat funktioniert nicht?

Mit `use_lets_encrypt = true` werden Zertifikate automatisch von Let's Encrypt geholt. Voraussetzungen:
- Port 80 muss von aussen erreichbar sein (fuer HTTP-01 Challenge)
- DNS muss korrekt auf den Server zeigen
- `production_domain` muss gesetzt sein (nicht `localhost`)

Zertifikate werden automatisch alle 24 Stunden geprueft und 30 Tage vor Ablauf erneuert.

Falls Let's Encrypt nicht geht, manuelle Zertifikate ablegen:

```
.rss/certs/example.com.fullchain.pem
.rss/certs/example.com.privkey.pem
```

### Fritz!Box: Zugriff funktioniert nur von extern?

Viele Router (inkl. Fritz!Box) unterstuetzen kein "NAT Loopback" — Zugriff auf die eigene oeffentliche IP aus dem internen Netzwerk wird blockiert.

Loesungen:
1. **DNS-Server am Rechner aendern:** Auf `8.8.8.8` (Google) oder `1.1.1.1` (Cloudflare) stellen, um den Fritz!Box-DNS-Cache zu umgehen
2. **Von extern testen:** Mobilfunk oder ein anderes Netzwerk verwenden
3. **Lokal direkt zugreifen:** `http://localhost:8000` funktioniert immer

### Hot Reload funktioniert nicht?

Hot Reload benoetigt eine WebSocket-Verbindung. Pruefen:
- Browser-Konsole (F12) auf WebSocket-Fehler pruefen
- Dateien muessen im richtigen Verzeichnis liegen: `www/{name}-[{port}]/`
- Unterstuetzte Dateitypen: HTML, CSS, JS, JSON, SVG, Bilder
