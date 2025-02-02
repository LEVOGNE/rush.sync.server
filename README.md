# Rush Sync - Terminal-basierte Synchronisationsanwendung

Rush Sync ist eine moderne, Terminal-basierte Anwendung, geschrieben in Rust. Sie bietet eine interaktive Benutzeroberfläche mit Echtzeit-Updates, Scrolling-Funktionalität und anpassbarer Konfiguration.

## Features

- 🎨 Anpassbares Farbschema
- ⌨️ Konfigurierbare Tastenbelegungen
- 📜 Scrollbare Nachrichtenansicht
- 💾 Befehlshistorie
- ⚡ Typewriter-Effekt für Nachrichten
- 🔄 Auto-Scroll Funktion
- 📝 Blinkender Cursor
- 🎯 Intuitive Tastaturnavigation

## Installation

### Voraussetzungen

- Rust (Edition 2021)
- Cargo

### Build-Prozess

```bash
# Repository klonen
git clone https://github.com/yourusername/rush-sync.git
cd rush-sync

# Anwendung bauen
cargo build --release

# Anwendung ausführen
cargo run --release
```

## Konfiguration

Die Anwendung kann über eine `rush.toml` Datei konfiguriert werden. Diese kann in folgenden Verzeichnissen platziert werden:

- `./rush.toml` (aktuelles Verzeichnis)
- `./config/rush.toml` (Produktionsumgebung)
- `./src/rush.toml` (Entwicklungsumgebung)

### Beispiel-Konfiguration

```toml
[general]
max_messages = 100
typewriter_delay = 50
input_max_length = 100
max_history = 30
poll_rate = 16

[theme]
input_text = "Yellow"
cursor = "Yellow"
output_text = "Green"
border = "DarkGray"

[prompt]
text = "/// "
color = "Yellow"
```

## Tastenbelegung

### Standard-Tastenbelegungen

- `←/→`: Cursor bewegen
- `Home/End`: Zum Anfang/Ende springen
- `Enter`: Eingabe bestätigen
- `Shift + ↑/↓`: Scrollen
- `Page Up/Down`: Seitenweise scrollen
- `↑/↓`: In der Historie navigieren
- `ESC` (2x): Anwendung beenden

## Entwicklung

### Projektstruktur

```
src/
├── message.rs     # Nachrichtenverwaltung
├── constants.rs   # Konstanten und Konfigurationspfade
├── scroll.rs      # Scroll-Funktionalität
├── error.rs       # Fehlerbehandlung
├── widget.rs      # Widget-Traits
├── config.rs      # Konfigurationsverwaltung
├── color.rs       # Farbverwaltung
├── terminal.rs    # Terminal-Setup
├── logging.rs     # Logging-System
├── screen.rs      # Bildschirm-Rendering
├── event.rs       # Event-Handling
├── keyboard.rs    # Tastatur-Input
├── cursor.rs      # Cursor-Verwaltung
├── input.rs       # Eingabeverarbeitung
└── main.rs        # Hauptanwendung
```

### Hauptkomponenten

1. **MessageManager**: Verwaltet die Nachrichtenliste und Scroll-Status
2. **ScreenManager**: Steuert das Terminal-Interface und Event-Handling
3. **InputState**: Verarbeitet Benutzereingaben und Historie
4. **Config**: Lädt und verwaltet die Anwendungskonfiguration
5. **EventHandler**: Asynchrone Event-Verarbeitung
6. **KeyboardManager**: Tastatureingaben und Bindings

## Logging

Die Anwendung unterstützt verschiedene Log-Level:

- ERROR: Kritische Fehler
- WARN: Warnungen
- INFO: Informative Nachrichten
- DEBUG: Debug-Informationen

Die Logs werden im Terminal mit entsprechender Farbkodierung angezeigt.

## Contribution

Beiträge sind willkommen! Bitte beachten Sie folgende Punkte:

1. Fork des Repositories
2. Feature-Branch erstellen
3. Änderungen committen
4. Pull Request erstellen

## Lizenz

[Ihre gewählte Lizenz]

## Credits

Entwickelt mit folgenden Rust-Crates:
- tokio
- crossterm
- ratatui
- serde
- log
- lazy_static

---

Dokumentation zuletzt aktualisiert: [Datum]