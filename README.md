# Rush Sync

Rush Sync ist eine moderne Terminal-basierte Benutzeroberfläche, die in Rust entwickelt wurde. Sie bietet ein interaktives UI mit Logging-Funktionalität, Typewriter-Effekten und scrollbarem Output.

## Features

- Interaktive Terminal-Benutzeroberfläche
- Farbcodierte Log-Ausgaben (ERROR, WARN, INFO, DEBUG)
- Scrollbare Nachrichtenhistorie
- Typewriter-Effekt für neue Nachrichten
- Eingabehistorie mit Pfeiltasten-Navigation
- Konfigurierbare Farbthemen über TOML-Datei
- Unicode-Unterstützung
- Cursor-Navigation und -Bearbeitung

## Tastenkombinationen

- `↑/↓`: Durch Eingabehistorie navigieren
- `←/→`: Cursor im Text bewegen
- `Home/End`: Zum Anfang/Ende der Zeile springen
- `Shift + ↑/↓`: Eine Zeile scrollen
- `Page Up/Down`: Seitenweise scrollen
- `Enter`: Eingabe bestätigen
- `ESC` (doppelt): Programm beenden

## Konfiguration

Die Konfiguration erfolgt über eine `rush.toml` Datei mit folgenden Hauptsektionen:

```toml
[general]
max_messages = 100      # Maximale Anzahl gespeicherter Nachrichten
typewriter_delay = 30   # Verzögerung des Typewriter-Effekts (ms)
input_max_length = 100  # Maximale Eingabelänge
max_history = 30        # Größe der Eingabehistorie
poll_rate = 16         # Event-Poll-Rate (ms)

[theme]
input_text = "White"    # Farbe des Eingabetexts
cursor = "White"       # Cursor-Farbe
output_text = "DarkGray" # Farbe des Ausgabetexts
border = "DarkGray"     # Rahmenfarbe

[prompt]
text = "/// "          # Eingabeaufforderung
color = "White"        # Farbe der Eingabeaufforderung
```

## Projektstruktur

```
src/
├── core/           # Kernfunktionalität
├── ui/            # UI-Komponenten
├── input/         # Eingabeverarbeitung
├── output/        # Ausgabeformatierung
└── setup/         # Konfiguration
```

## Technische Details

- Asynchrone Architektur mit Tokio
- Event-basiertes System für Eingabehandlung
- Modulares Design mit klarer Trennung der Verantwortlichkeiten
- Cross-Platform Terminal-Handling mit Crossterm
- TUI-Rendering mit Ratatui

## Abhängigkeiten

- tokio (async runtime)
- crossterm (terminal handling)
- ratatui (terminal user interface)
- serde + toml (Konfiguration)
- log (Logging-Framework)
- unicode-segmentation (Unicode-Support)

## Build & Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## Entwicklung

Das Projekt verwendet eine modulare Struktur für einfache Erweiterbarkeit. Neue Features können durch Implementierung entsprechender Traits hinzugefügt werden:

- `Widget` für neue UI-Komponenten
- `InputWidget` für Eingabe-Handler
- Erweiterung der `KeyAction` Enum für neue Tastenkombinationen

## Lizenz

MIT License

## Beiträge

Beiträge sind willkommen! Bitte erstellen Sie einen Pull Request oder ein Issue für Verbesserungsvorschläge.