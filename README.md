# Rush Sync

Rush Sync ist eine moderne Terminal-basierte Benutzeroberfläche, die in Rust entwickelt wurde. Sie bietet ein interaktives UI mit Logging-Funktionalität, Typewriter-Effekten, scrollbarem Output und anpassbarem Design.

## Features

- Interaktive Terminal-Benutzeroberfläche
- Farbcodierte Log-Ausgaben (ERROR, WARN, INFO, DEBUG)
- Scrollbare Nachrichtenhistorie mit Auto-Scroll
- Typewriter-Effekt für neue Nachrichten
- Eingabehistorie mit Pfeiltasten-Navigation
- Vollständig anpassbares Design über TOML-Konfiguration
- Hintergrund- und Vordergrundfarben für Input/Output
- Unicode-Unterstützung
- Erweiterte Cursor-Navigation und -Bearbeitung
- Automatische Konfigurationserstellung

## Tastenkombinationen

- `↑/↓`: Durch Eingabehistorie navigieren
- `←/→`: Cursor im Text bewegen
- `Home/End`: Zum Anfang/Ende der Zeile springen
- `Shift + ↑/↓`: Eine Zeile scrollen
- `Page Up/Down`: Seitenweise scrollen
- `Enter`: Eingabe bestätigen
- `ESC` (doppelt): Programm beenden
- `Backspace/Delete`: Text löschen

## Konfiguration

Die Konfiguration erfolgt über eine `rush.toml` Datei. Diese wird automatisch im `.rss` Verzeichnis neben der ausführbaren Datei erstellt.

### Standard-Konfiguration

```toml
[general]
max_messages = 100      # Maximale Anzahl gespeicherter Nachrichten
typewriter_delay = 30   # Verzögerung des Typewriter-Effekts (ms)
input_max_length = 100  # Maximale Eingabelänge
max_history = 30        # Größe der Eingabehistorie
poll_rate = 16         # Event-Poll-Rate (ms)

[theme]
input_text = "Black"    # Farbe des Eingabetexts
input_bg = "White"     # Hintergrundfarbe des Eingabebereichs
cursor = "Black"       # Cursor-Farbe
output_text = "DarkGray" # Farbe des Ausgabetexts
output_bg = "Black"    # Hintergrundfarbe des Ausgabebereichs

[prompt]
text = "/// "          # Eingabeaufforderung
color = "Black"        # Farbe der Eingabeaufforderung
```

### Verfügbare Farben

- Standard: `Black`, `White`, `Gray`, `DarkGray`
- Primärfarben: `Red`, `Green`, `Blue`, `Yellow`, `Magenta`, `Cyan`
- Helle Varianten: `LightRed`, `LightGreen`, `LightBlue`, `LightYellow`, `LightMagenta`, `LightCyan`

## Installation

### Voraussetzungen

- Rust/Cargo (neueste stabile Version)
- Git (optional, für Entwicklung)

### Build von Source

```bash
# Repository klonen (optional)
git clone https://github.com/username/rush_sync.git
cd rush_sync

# Build
cargo build --release

# Ausführen
cargo run --release
```

## Projektstruktur

```
src/
├── core/           # Kernfunktionalität
│   ├── config.rs   # Konfigurationshandling
│   ├── constants.rs # Konstanten
│   ├── error.rs    # Fehlertypen
│   └── prelude.rs  # Common Imports
├── ui/            # UI-Komponenten
│   ├── widget.rs   # Widget-Traits
│   ├── color.rs    # Farbhandling
│   ├── cursor.rs   # Cursor-Logik
│   └── screen.rs   # Hauptscreen-Rendering
├── input/         # Eingabeverarbeitung
│   ├── event.rs    # Event-Handling
│   ├── keyboard.rs # Tastatur-Input
│   └── input.rs    # Eingabe-Widget
├── output/        # Ausgabeformatierung
│   ├── message.rs  # Nachrichtenhandling
│   ├── logging.rs  # Logging-System
│   └── scroll.rs   # Scroll-Logik
└── setup/         # Konfiguration & Setup
    └── setup_toml.rs # TOML Setup
```

## Technische Details

### Architektur

- **Event Loop**: Asynchrone Event-Verarbeitung mit Tokio
- **Terminal Handling**: Cross-Platform mit Crossterm
- **UI Rendering**: Modernes TUI-Framework Ratatui
- **Konfiguration**: TOML-basiert mit Serde
- **Logging**: Flexibles Logging-System mit verschiedenen Levels
- **Unicode**: Volle Unicode-Unterstützung mit Grapheme-Clusters

### Hauptkomponenten

- **ScreenManager**: Zentrale UI-Komponente
- **MessageManager**: Nachrichtenverwaltung und Scrolling
- **InputState**: Eingabeverarbeitung und Historie
- **ScrollState**: Scroll-Position und Auto-Scroll
- **AppLogger**: Asynchrones Logging-System

## Performance

- Effizientes Memory-Management durch Ringpuffer
- Optimierte Render-Zyklen
- Asynchrone Event-Verarbeitung
- Minimaler CPU-Verbrauch im Idle

## Dependencies

```toml
[dependencies]
crossterm = "0.27"
ratatui = "0.24"
unicode-segmentation = "1.10"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
env_logger = "0.10"
log = "0.4"
lazy_static = "1.4"
strip-ansi-escapes = "0.1.1"
tokio = { version = "1.36", features = ["full"] }
futures = "0.3"
dirs = "5.0"
```

## Entwicklung

### Code-Konventionen

- Rust 2021 Edition
- Volle Dokumentation aller öffentlichen APIs
- Fehlerbehandlung mit eigenen Error-Typen
- Modulare Struktur mit klaren Zuständigkeiten

### Testing

```bash
# Unit Tests ausführen
cargo test

# Integration Tests
cargo test --test '*'

# Mit Logging
RUST_LOG=debug cargo test
```

### Debugging

- Integriertes Debug-Logging
- Konfigurierbare Log-Level
- Detaillierte Cursor-Operation-Logs

## Bekannte Einschränkungen

- Minimale Terminalgrößen-Anforderung: 20x10 Zeichen
- Keine Mausunterstützung
- Keine RTL-Sprachen-Unterstützung

## Roadmap

- [ ] Mausunterstützung
- [ ] Split-Screen-Modus
- [ ] Syntax-Highlighting
- [ ] Plugin-System
- [ ] Konfiguration über UI
- [ ] Verbesserte Unicode-Unterstützung

## Lizenz

MIT License - Siehe [LICENSE](LICENSE) Datei

## Beiträge

Beiträge sind willkommen! Bitte beachten Sie:

1. Fork des Repositories
2. Feature Branch erstellen
3. Änderungen committen
4. Tests hinzufügen/anpassen
5. Pull Request erstellen
