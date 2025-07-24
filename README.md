# Rush Sync

**Rush Sync** ist eine moderne, modulare Terminal-Anwendung in **Rust**, mit interaktiver TUI, internationalisierter Oberfläche, farbcodiertem Logging und flexibler Konfiguration.  
Ideal für Entwickler, die eine anpassbare, skriptfähige Terminal-UI benötigen.

---

## ✅ Features

- **Interaktive Terminal-UI** mit modernem Design  
- **Farbcodierte Log-Ausgaben** (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`)  
- **Typewriter-Effekt** für neue Nachrichten  
- **Scrollbare Nachrichtenhistorie** mit Auto-Scroll  
- **Eingabehistorie** & erweiterte Cursor-Navigation  
- **Konfigurierbares Design** via TOML (Farben, Prompt, Layout)  
- **Automatische Konfigurationserstellung**  
- **Mehrsprachigkeit (i18n)** integriert  
- **Unicode-Support (Grapheme-basiert)**

---

## ⌨️ Tastenkombinationen

| Taste | Funktion |
|-------|---------|
| `↑/↓` | Eingabehistorie navigieren |
| `←/→` | Cursor im Text bewegen |
| `Home / End` | Zum Anfang / Ende springen |
| `Shift + ↑/↓` | Zeilenweise scrollen |
| `Page Up / Down` | Seitenweise scrollen |
| `Enter` | Eingabe bestätigen |
| `ESC` (doppelt) | Programm beenden |
| `Backspace / Delete` | Zeichen löschen |

---

## ⚙️ Konfiguration

Die **`rush.toml`** wird beim ersten Start automatisch im `.rss`-Verzeichnis erstellt.

### Standard-Config

```toml
[general]
max_messages = 100
typewriter_delay = 30
input_max_length = 100
max_history = 30
poll_rate = 16

[theme]
input_text = "Black"
input_bg = "White"
cursor = "Black"
output_text = "DarkGray"
output_bg = "Black"

[prompt]
text = "/// "
color = "Black"
```

### Farben

`Black`, `White`, `Gray`, `DarkGray`, `Red`, `Green`, `Blue`, `Yellow`,  
`Magenta`, `Cyan`, `LightRed`, `LightGreen`, `LightBlue`, `LightYellow`,  
`LightMagenta`, `LightCyan`

---

## 🚀 Installation

### Voraussetzungen

- **Rust** (2021 Edition, stabile Version)  
- **Cargo** (automatisch bei Rust enthalten)  
- Git (optional)

### Build & Run

```bash
git clone https://github.com/username/rush_sync.git
cd rush_sync
cargo build --release
cargo run --release
```

---

## 🗂 Projektstruktur (vereinfacht)

```
src/
├── core/        # Kernlogik (Config, Error, Constants, Prelude)
├── ui/          # Terminal-UI (ScreenManager, TerminalManager, Widgets)
├── input/       # Eingabe-Handling (Keyboard, EventLoop)
├── output/      # Logging & Scrolling
├── commands/    # Integrierte Commands (exit, clear, history, lang, version)
└── setup/       # TOML-Setup & Autokonfiguration
```

---

## 🛠 Technische Details

- **Event Loop**: Tokio (async)  
- **Terminal**: Crossterm + Ratatui  
- **Logging**: Eigenes asynchrones Logging-System  
- **Config**: Serde + TOML  
- **Unicode**: Voller Grapheme-Support  
- **Commands**: Modulares Trait-basiertes System

---

## 🧪 Testing & Debugging

```bash
cargo test
RUST_LOG=debug cargo test
```

---

## ⚠ Bekannte Einschränkungen

- Min. Terminalgröße: **20x10** Zeichen  
- Keine Mausunterstützung (geplant)  
- Kein RTL-Support

---

## 🗺 Roadmap

- [ ] Mausunterstützung  
- [ ] Split-Screen & Tabs  
- [ ] Syntax-Highlighting  
- [ ] Plugin-System  
- [ ] Konfiguration direkt aus der UI  
- [ ] Erweiterte Unicode-Unterstützung

---

## 📜 Lizenz

MIT License – siehe [LICENSE](LICENSE)

---

## 🤝 Beiträge

1. Fork erstellen  
2. Feature-Branch anlegen  
3. Änderungen + Tests committen  
4. Pull Request erstellen
