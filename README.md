# RUSH SYNC

Ein leistungsfähiges Terminal-basiertes Synchronisations- und Messaging-Tool mit Unterstützung für anpassbare Konfigurationen, Farbthemen und Echtzeit-Nachrichtenverwaltung.

## 📌 Features

- **Message Management**: Nachrichtenverlauf mit Auto-Scroll und begrenzter Historie.
- **Scroll-System**: Unterstützung für Seitenscrolling und dynamische Anpassung.
- **Konfigurationsdatei** (`rush.toml`): Anpassbare Einstellungen für Nachrichtenanzahl, Farbschema und Eingabeverhalten.
- **Logging-System**: Strukturierte Logs mit verschiedenen Levels (`INFO`, `DEBUG`, `ERROR`).
- **Asynchrones Event-Handling** mit `tokio`.
- **Typewriter-Effekt** für das langsame Anzeigen von Nachrichten.
- **Tastatursteuerung** für einfache Navigation im Terminal.

## 📂 Projektstruktur

```
├── src/
│   ├── main.rs          # Einstiegspunkt der Anwendung
│   ├── message.rs       # Nachrichtensystem
│   ├── constants.rs     # Globale Konstanten
│   ├── scroll.rs        # Scroll-Logik
│   ├── error.rs         # Fehlerhandling
│   ├── widget.rs        # Basis-Widgets für das Terminal-UI
│   ├── config.rs        # Konfigurationsverwaltung
│   ├── color.rs         # Farbmanagement
│   ├── terminal.rs      # Terminalsteuerung
│   ├── logging.rs       # Logging-Mechanismus
│   ├── screen.rs        # Bildschirmmanagement
│   ├── event.rs         # Event-Handling
│   ├── keyboard.rs      # Tastatureingaben und Hotkeys
│   ├── output.rs        # Terminal-Ausgabeformatierung
│   ├── input.rs         # Eingabe-Management
│   ├── prelude.rs       # Sammelmodul für Importe
│   └── rush.toml        # Konfigurationsdatei
```

## 🛠 Installation & Nutzung

### 1️⃣ **Projekt kompilieren und ausführen**

```sh
cargo run
```

### 2️⃣ **Optimierte Version (Release-Build)**

```sh
cargo build --release
./target/release/rush_sync
```

### 3️⃣ **Konfigurationsdatei (`rush.toml`) anpassen**

Falls `rush.toml` nicht existiert, wird eine Standardkonfiguration geladen.

## 🎮 Tastenkombinationen

| Tastenkombination | Aktion                 |
| ----------------- | ---------------------- |
| `↑ / ↓`           | Verlauf durchblättern  |
| `Seite ↑ / ↓`     | Seitenscrolling        |
| `Enter`           | Eingabe absenden       |
| `ESC (zweimal)`   | Beenden                |
| `ALT + ↑ / ↓`     | Scrollen um eine Zeile |

## ✏️ Verbesserungen in der Resize-Logik

Ein Problem mit der Resize-Logik in `screen.rs` wurde behoben. Die wichtigsten Verbesserungen:

- **Robuste Größenvalidierung:**

  - Einführung von konstanten Mindestgrößen (`MIN_WIDTH = 20`, `MIN_HEIGHT = 10`)
  - Fallback-Rendering bei zu kleinem Terminal
  - Benutzerfreundliche Fehlermeldung

- **Verbesserte ScrollState-Logik:**

  - Beibehaltung der relativen Scroll-Position bei Größenänderungen
  - Intelligentere Behandlung von Auto-Scroll
  - Verbesserte Offset-Berechnung

- **Zuverlässigeres Rendering:**

  - Sicherheitsüberprüfungen vor dem Rendering
  - Optimierte Layout-Berechnung
  - Korrekte Behandlung der verfügbaren Höhe

- **Besseres Debugging:**
  - Ausführlicheres Logging für Resize-Events
  - Nachverfolgbarkeit von Größenänderungen
  - Klare Fehlermeldungen

Mit diesen Verbesserungen sollte die Anwendung stabiler auf Terminalgrößenänderungen reagieren und ein fehlerfreies Rendering ermöglichen.

## 📝 Lizenz

Dieses Projekt steht unter der **MIT-Lizenz**.

---

🚀 Entwickelt mit **Rust**, `tokio`, `crossterm` und `ratatui`.
