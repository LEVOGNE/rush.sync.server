# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)

**Rush Sync Server** ist eine moderne, modulare Terminal-Anwendung in **Rust** mit interaktiver TUI, internationalisierter Oberfläche, farbcodiertem Logging und flexibler Konfiguration.
Ideal für Entwickler, die eine **anpassbare, skriptfähige Terminal-UI** suchen.

---

## ✅ Features

- **Interaktive Terminal-UI** mit asynchronem Eventloop (Tokio)
- **Farbcodiertes Logging** mit Level-Erkennung (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`)
- **Internationalisierung (i18n):**
  - Mehrsprachige Marker werden automatisch auf Standard-Farben gemappt (`[SPRACHE]`, `[IDIOMA]` → `lang` → Cyan)
  - Dynamische Sprachumschaltung im laufenden Betrieb
- **Typewriter-Effekt** & **Blinkender Cursor**
- **Auto-Scroll & Scrollbare Nachrichtenhistorie**
- **Eingabehistorie** & volle Cursor-Navigation
- **Modularer Commands-Handler** (`exit`, `lang`, `restart`, `version`, etc.)
- **Konfigurierbares Design & Prompt** via TOML
- **Unicode-Support (Grapheme-basiert)**
- **Restart-Funktion** ohne externen Neustart

---

## ⌨️ Tastenkombinationen

| Taste            | Funktion                         |
| ---------------- | -------------------------------- |
| `↑ / ↓`          | Eingabehistorie navigieren       |
| `← / →`          | Cursor im Text bewegen           |
| `Home / End`     | Zum Anfang / Ende springen       |
| `Shift + ↑ / ↓`  | Zeilenweise scrollen             |
| `Page Up / Down` | Seitenweise scrollen             |
| `Enter`          | Eingabe bestätigen               |
| `ESC` (doppelt)  | Programm beenden                 |
| `__RESTART__`    | Internen Neustart (Kalt-Restart) |
| `__CLEAR__`      | Nachrichten leeren               |

---

## ⚙️ Konfiguration

Die **`rush.toml`** wird automatisch im `.rss`-Verzeichnis erstellt.

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

### Farben (COLOR_MAP)

Unterstützt werden:
`Black`, `White`, `Gray`, `DarkGray`, `Red`, `Green`, `Blue`, `Yellow`,
`Magenta`, `Cyan`, `LightRed`, `LightGreen`, `LightBlue`, `LightYellow`,
`LightMagenta`, `LightCyan`

i18n-Übersetzungen werden automatisch auf Standard-Keys gemappt
(z.B. `"Sprache"`, `"Idioma"`, `"Язык"` → `lang` → Cyan).

---

## 🚀 Installation

### Voraussetzungen

- **Rust** (2021 Edition, stable)
- **Cargo** (enthalten bei Rust)
- Git (optional)

### Build & Run

```bash
git clone https://github.com/username/rush_sync.git
cd rush_sync
cargo build --release
cargo run --release
```

---

## 🗂 Projektstruktur

```graphql
src/
├── core/        # Kernlogik (Config, Error, Prelude)
├── ui/          # Terminal-UI (ScreenManager, TerminalManager, Widgets)
├── input/       # Input-Handling (Keyboard, EventHandler)
├── output/      # Logging, MessageManager, Color
├── commands/    # Modulare Commands (exit, lang, history, restart)
└── setup/       # Autokonfiguration (TOML-Setup)
```

---

## 🛠 Technische Details

- **Eventloop:** Asynchron (Tokio) → gesplittet in
  - `handle_input_event`
  - `handle_tick_event`
  - `handle_resize_event`
- **Logging:**
  - Globaler `AppLogger` (Intercept für alle `log::*` Calls)
  - `LogMessage` speichert Level + Text → farbcodierte Anzeige
- **Internationalisierung:**
  - `get_marker_color` mappt automatisch übersetzte Marker auf Standardkategorien
- **Restart:** Intern, ohne externen Programmneustart

---

## 🧪 Testing

```bash
cargo test
RUST_LOG=debug cargo test
```

Tests vorhanden für:
✔ Commands
✔ Eventloop
✔ Config-Setup
✔ i18n-Übersetzungen

---

## 🗺 Roadmap

- [ ] Mausunterstützung (Scroll & Auswahl)
- [ ] Split-Screen & Tabs
- [ ] Syntax-Highlighting
- [ ] Plugin-System für eigene Commands
- [ ] UI-Konfiguration live ändern

---

## 📜 Lizenz

### **Dual-Lizenz-Modell**

Dieses Projekt steht unter einer **Dual-Lizenz**:

1. **Community License (GPLv3)** – frei für private und nicht-kommerzielle Nutzung.
   Siehe [LICENSE](LICENSE).
2. **Commercial License** – erforderlich für jede kommerzielle Nutzung.
   Siehe [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md).

**Kontakt für kommerzielle Lizenzen:**
📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)

---

## 🤝 Beiträge

1. Fork erstellen
2. Feature-Branch anlegen
3. Änderungen + Tests committen
4. Pull Request erstellen
