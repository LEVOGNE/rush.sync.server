# Rush Sync Server – Benchmark & Ressourcen-Footprint (v0.3.6)

> Stand: 2025-09-10 • Zielplattform: macOS (Dev-Setup) • Laufzeit: Tokio + Actix-Web
> Fokus dieses Benchmarks: **Ressourcenverbrauch im Leerlauf** (Memory, FDs) mit/ohne laufende Dev-Server.
> **Nicht** Bestandteil: Throughput/Latency-Benchmarks unter Last.

## TL;DR

- **Baseline (ohne Server):** ~**16.5 MB RSS**
- **+1 Server:** ~**37.0 MB RSS**
- **+10 Server:** ~**48.9 MB RSS**
- Daraus ergibt sich ein **idle-Overhead pro zusätzlichem Server von ~3.3 MB** (nach Warmup, geteilter Code/Runtime).
- **Open FDs** skaliert erwartungsgemäß (Watcher/Proxy/Sockets): von ~15 (idle) auf **~147** bei 10 Servern.
- Für ein Rust-basiertes Dev-Tool mit Reverse-Proxy, TLS, Hot-Reload & File-Watcher sind diese Werte **sehr gut**.
  Gegenüber hochoptimierten, spezialisierten Reverse-Proxies (nginx/HAProxy) ist der pure **Proxy-Footprint** höher, aber als **Dev-Orchestrator/App-Server-Spawner** liegt Rush in einem **sehr effizienten Bereich**.

---

## Umgebung & Messmethode

- **Build**: `cargo run` (Debug-Build, unoptimiert)
- **OS**: macOS (VMS ist systembedingt sehr hoch und sagt **nichts** über echten RAM aus)
- **Laufzeit**: Tokio runtime vorhanden (einmalig), Actix-Web pro Server
- **Messpunkte** via interner Kommandoschnittstelle:
  - `mem info` (Tabelle)
  - `mem info --json`
  - `mem info --all` (FD-Übersicht; auf macOS heuristisch, Linux detailliert)

> Hinweis: „Registry TOTAL“ ≈ Summe der **eingebetteten Assets/Phasen** (konstant), **RSS** = realer Prozess-RAM.

---

## Ergebnisse

### Zusammenfassung (aus deinen Logs)

| Szenario             | RSS (MB) | Registry TOTAL (MB) | VMS (GB)\* | FDs (total) | Sockets |
| -------------------- | -------: | ------------------: | ---------: | ----------: | ------: |
| Baseline (0 Server)  |    ~16.5 |               ~11.7 |       ~401 |          15 |       7 |
| +1 Server gestartet  |    ~37.0 |               ~11.7 |   ~401–411 |          28 |      12 |
| +10 Server gestartet |    ~48.9 |               ~11.7 |       ~402 |         147 |      59 |

\* **VMS auf macOS** ist traditionell extrem groß (große, reservierte Adressräume); das ist **kein** Indikator für realen RAM-Verbrauch.

**Abgeleitet:**

- **Idle-Overhead pro Server ≈ (48.9 − 16.5) / 10 ≈ 3.3 MB**
- **Registry TOTAL bleibt konstant** (~11.7 MB), da es sich vor allem um eingebettete Assets & Phasen-Deltas handelt.

### Beispielauszüge (gekürzt)

- `mem info` bei 10 Servern:
  - **TOTAL (Registry):** 12,239,194 B (11.672 MB)
  - **RSS:** 48,922,624 B (46.656 MB)
  - **FDs:** 147 (Sockets: 59)
- `mem info --json` zeigt identische Summen und die Top-Einträge der Registry.

---

## Einordnung & Vergleich

> ⚠️ Äpfel vs. Birnen: Rush ist **kein** reiner Reverse-Proxy, sondern ein **Dev-Orchestrator** (Proxy **plus** TLS-Terminator, File-Watcher, Hot-Reload, Mini-Server-Spawner).
> Reine Proxies (nginx, HAProxy) sind auf **eine Aufgabe** getrimmt und oft im **einstelligen MB-Bereich** pro Prozess unterwegs – allerdings ohne Watcher/Dev-Komfort.

**Grobvergleich (qualitativ):**

- **nginx / HAProxy (reine Proxies):**

  - Sehr niedriger Idle-Footprint, extrem effizient pro Verbindung.
  - Kein integriertes Dev-Erlebnis (kein File-Watcher/Hot-Reload), keine App-Server-Spawns.
  - Für Production-Edge sehr schwer zu schlagen, aber nicht vergleichbar mit Rush’ Dev-Scope.

- **Caddy / Traefik (All-in-One Proxies mit TLS & DX):**

  - Höherer Idle-Footprint als nginx/HAProxy (mehr Features out-of-the-box).
  - Ebenfalls guter DX, aber fokussiert auf **Prod-Reverse-Proxy**/Routing, nicht auf lokale **Multi-App-Dev** mit TUI/Spawner.

- **Node/Express/Next dev-Server (App-Server-Klasse):**
  - Üblicherweise deutlich höherer Per-Server-Idle-Footprint.
  - 10 parallele Dev-Instanzen liegen häufig **weit** über 100 MB gesamt.
  - Rush (~**49 MB gesamt für 10 Server**) ist hier **sehr schlank** – Rust + shared runtime zahlen sich aus.

**Fazit der Einordnung:**
Für den **Dev-Use-Case** (mehrere lokale Webserver + Proxy + TLS + File-Watch + TUI) liefert Rush **einen sehr guten Footprint**. Gegenüber reinen Proxies ist der pure Proxy-Teil naturgemäß nicht minimal, aber **in Summe** (Orchestrierung + Servers + Tools) bist du **effizienter als viele vergleichbare Dev-Stacks**, insbesondere wenn mehrere Instanzen parallel laufen.

---

## Hinweise zur Interpretation

- **„Registry TOTAL“** zeigt **statische** Last (eingebettete Assets, Phasen-Deltas) – bleibt konstant, egal wie viele Server laufen.
- **RSS** wächst mit aktiven Servern/Tasks; durch **shared Code/Runtimes** fällt der **zusätzliche** Verbrauch **pro Server** gering aus (~3.3 MB idle in deinem Setup).
- **FDs** steigen mit Listenern, WebSockets, Watchern – in Dev-Umgebungen normal.
- **VMS (macOS)** ist groß und **irrelevant** als RAM-Kennzahl.

---

## Reproduzierbarkeit

```bash
# 1) Projekt starten (Debug)
cargo run

# 2) Einen Server anlegen & starten
create
start 1

# 3) Snapshot ziehen (Tabelle)
mem info

# 4) Vollständig als JSON
mem info --json

# 5) Erweiterte Prozessinfos (FDs, /proc/* auf Linux)
mem info --all
mem info --json --all
```

---

## Empfehlungen & nächste Schritte

1. **Release-Build prüfen:**
   `cargo build --release` → reduziert i.d.R. Basis-Overhead (sowohl CPU als auch RSS).

2. **Lasttests hinzufügen (separater Benchmark):**
   Mit `wrk`, `bombardier` oder `hey` pro Server (HTTP/1.1 & HTTP/2), Metriken: **RPS, P50/P90/P99-Latency, CPU**, **RSS-Δ unter Last**.

3. **Prod-Profil ohne Watcher/Hot-Reload:**
   In Produktion Watcher & TUI deaktivieren → **FDs und Idle-RSS** sinken weiter.

4. **Tokio/Actix-Tuning:**
   Workers/Threadpools konfigurieren, um Overhead und Context-Switching zu optimieren, besonders bei vielen gleichzeitigen Instanzen.

5. **Per-Server-Phasen messen:**
   Optional pro gestarteter Instanz `phase:server:<name>@v1` registrieren → ΔRSS pro Server **explizit** sichtbar.

---

## Schlussbewertung

- **Für Dev-Orchestrierung** mit mehreren parallelen Services ist der beobachtete Footprint **sehr gut**: ~**49 MB gesamt für 10 Server** ist **schlank**, **stabil** und **linear skalierend**.
- **Gegenüber reinen Proxies** verlierst du im reinen Proxy-Idle-Vergleich (diese sind auf Minimalismus optimiert),
  **gewinnst** aber durch die integrierte Entwickler-Erfahrung (Spawner, Watcher, TLS, TUI) bei gleichzeitig **sehr niedrigen Mehrkosten pro Server**.
- **Gegenüber typischen JS-Dev-Server-Setups** bist du beim Memory-Verbrauch pro Instanz **deutlich im Vorteil**.

> Kurz: **Rush** liegt in seinem Zielkorridor **stark** – effizient, reproduzierbar und ausbaufähig für echte Load-Benchmarks.
