# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)

**Rush Sync Server** is a modern, modular terminal application written in **Rust**, featuring an interactive TUI, internationalized interface, color-coded logging, and flexible configuration.
Perfect for developers who need a **customizable, scriptable terminal UI**.

---

## ✅ Features

- **Interactive terminal UI** with an asynchronous event loop (Tokio)
- **Color-coded logging** with level detection (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`)
- **Internationalization (i18n):**
  - Multilingual markers are automatically mapped to standard colors (`[SPRACHE]`, `[IDIOMA]` → `lang` → Cyan)
  - Dynamic language switching at runtime
- **Typewriter effect** & **blinking cursor**
- **Auto-scroll & scrollable message history**
- **Input history** & full cursor navigation
- **Modular command handler** (`exit`, `lang`, `restart`, `version`, etc.)
- **Configurable design & prompt** via TOML
- **Unicode support (grapheme-based)**
- **Restart function** without external process restart

---

## ⌨️ Keyboard Shortcuts

| Key              | Function                        |
| ---------------- | ------------------------------- |
| `↑ / ↓`          | Navigate input history          |
| `← / →`          | Move cursor in text             |
| `Home / End`     | Jump to start / end of input    |
| `Shift + ↑ / ↓`  | Scroll line by line             |
| `Page Up / Down` | Scroll page by page             |
| `Enter`          | Confirm input                   |
| `ESC` (twice)    | Exit the program                |
| `__RESTART__`    | Internal restart (cold restart) |
| `__CLEAR__`      | Clear all messages              |

---

## ⚙️ Configuration

The **`rush.toml`** file is automatically created in the `.rss` directory on first start.

### Default Configuration

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

### Colors (COLOR_MAP)

Supported:
`Black`, `White`, `Gray`, `DarkGray`, `Red`, `Green`, `Blue`, `Yellow`,
`Magenta`, `Cyan`, `LightRed`, `LightGreen`, `LightBlue`, `LightYellow`,
`LightMagenta`, `LightCyan`

i18n translations are automatically mapped to standard keys
(e.g., `"Sprache"`, `"Idioma"`, `"Язык"` → `lang` → Cyan).

---

## 🚀 Installation

### Requirements

- **Rust** (2021 Edition, stable)
- **Cargo** (included with Rust)
- Git (optional)

### Build & Run

```bash
git clone https://github.com/username/rush_sync.git
cd rush_sync
cargo build --release
cargo run --release
```

---

## 🗂 Project Structure

```graphql
src/
├── core/        # Core logic (Config, Error, Prelude)
├── ui/          # Terminal UI (ScreenManager, TerminalManager, Widgets)
├── input/       # Input handling (Keyboard, EventHandler)
├── output/      # Logging, MessageManager, Color
├── commands/    # Modular commands (exit, lang, history, restart)
└── setup/       # Auto-configuration (TOML setup)
```

---

## 🛠 Technical Details

- **Event loop:** Asynchronous (Tokio) → split into:
  - `handle_input_event`
  - `handle_tick_event`
  - `handle_resize_event`
- **Logging:**
  - Global `AppLogger` (intercepts all `log::*` calls)
  - `LogMessage` stores level + text → color-coded output
- **Internationalization:**
  - `get_marker_color` automatically maps translated markers to standard categories
- **Restart:** Internal, without external process restart

---

## 🧪 Testing

```bash
cargo test
RUST_LOG=debug cargo test
```

Available tests:
✔ Commands
✔ Event loop
✔ Config setup
✔ i18n translations

---

## 🗺 Roadmap

- [ ] Mouse support (scroll & selection)
- [ ] Split-screen & tabs
- [ ] Syntax highlighting
- [ ] Plugin system for custom commands
- [ ] Live UI configuration changes

---

## 📜 License

### **Dual-Licensing Model**

This project is distributed under a **dual license**:

1. **Community License (GPLv3)** – free for private and non-commercial use.
   See [LICENSE](LICENSE).
2. **Commercial License** – required for any commercial use.
   See [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md).

**Contact for commercial licensing:**
📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)

---

## 🤝 Contributing

1. Fork this repository
2. Create a feature branch
3. Commit changes + add tests
4. Submit a pull request
