# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> 🛠 **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.2.3+`** for a stable release!

**Rush Sync Server** is a modern, modular terminal application written in **Rust**, featuring an interactive TUI, internationalized interface, color-coded logging, and flexible configuration.
Perfect for developers who need a **customizable, scriptable terminal UI**.

---

## 🚀 Installation & Usage

### 📦 **As Binary (End Users) - Version 0.2.3+**

```bash
# Install directly from crates.io
cargo install rush-sync-server

# Run the application
rush-sync
```

### 📚 **As Library (Developers) - Version 0.2.3+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.2.3"
tokio = { version = "1.36", features = ["full"] }
```

#### **Quick Start Examples:**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Option 1: Run with default configuration
    run().await?;
    Ok(())
}
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Option 2: Load config and run
    let config = load_config().await?;
    run_with_config(config).await?;
    Ok(())
}
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Option 3: Use as command handler only
    let handler = create_handler();

    let result = handler.handle_input("version");
    println!("Result: {}", result.message);

    let result = handler.handle_input("perf");
    println!("Performance: {}", result.message);

    Ok(())
}
```

#### **Advanced Library Usage:**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Custom configuration
    let mut config = load_config().await?;
    config.poll_rate = std::time::Duration::from_millis(8); // 125 FPS
    config.typewriter_delay = std::time::Duration::from_millis(1); // Ultra-fast

    // Run with custom screen manager
    let mut screen = ScreenManager::new(&config).await?;
    screen.run().await?;

    Ok(())
}
```

### 🛠 **From Source (Development)**

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
cargo build --release

# Run as binary
cargo run --release

# Or install locally
cargo install --path .
rush-sync
```

---

## ✅ Features

- **Interactive terminal UI** with an asynchronous event loop (Tokio)
- **Color-coded logging** with level detection (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`)
- **Runtime log-level switching** with persistent config saving
- **Performance monitoring** with real-time FPS & config analysis
- **Internationalization (i18n):**
  - Multilingual markers are automatically mapped to standard colors (`[SPRACHE]`, `[IDIOMA]` → `lang` → Cyan)
  - Dynamic language switching at runtime (German/English)
  - Automatic language detection and config persistence
- **Typewriter effect** & **blinking cursor** (configurable/disableable)
- **Auto-scroll & scrollable message history**
- **Input history** & full cursor navigation
- **Modular command handler** with extensible plugin system
- **Configurable design & prompt** via TOML with performance optimization
- **Unicode support (grapheme-based)**
- **Restart function** without external process restart
- **Smart bounds checking** with automatic config correction

---

## 💻 Available Commands

| Command                | Description                                     | Examples                         |
| ---------------------- | ----------------------------------------------- | -------------------------------- |
| `version` / `ver`      | Show application version                        | `version`                        |
| `lang` / `language`    | Switch language (EN/DE) with config persistence | `lang de`, `lang en`             |
| `clear` / `cls`        | Clear all messages                              | `clear`                          |
| `exit` / `q`           | Exit with confirmation                          | `exit`                           |
| `restart`              | Internal restart (reloads config)               | `restart`, `restart --force`     |
| `history -c`           | Clear input history                             | `history -c`                     |
| `log-level`            | Change log level (runtime + persistent)         | `log-level 3`, `log-level debug` |
| `perf` / `performance` | Show performance & config status                | `perf`, `performance`            |

### Log-Level Command Details

```bash
log-level           # Show current level
log-level 3         # Set to INFO level
log-level DEBUG     # Set to DEBUG level
log-level -h        # Show help

# Available levels:
# 1 = ERROR   (Only critical errors)
# 2 = WARN    (Warnings and errors)
# 3 = INFO    (General information) [DEFAULT]
# 4 = DEBUG   (Debug information)
# 5 = TRACE   (Very detailed tracing)
```

### Performance Command Details

```bash
perf                # Show full performance report
performance         # Same as perf
stats               # Same as perf
perf -h             # Show help
```

**Performance Report includes:**

- Current FPS (based on poll_rate)
- Typewriter speed (chars/second)
- Config values & file location
- Performance recommendations
- Related commands

---

## ⌨️ Keyboard Shortcuts

| Key              | Function                     |
| ---------------- | ---------------------------- |
| `↑ / ↓`          | Navigate input history       |
| `← / →`          | Move cursor in text          |
| `Home / End`     | Jump to start / end of input |
| `Shift + ↑ / ↓`  | Scroll line by line          |
| `Page Up / Down` | Scroll page by page          |
| `Enter`          | Confirm input                |
| `ESC` (twice)    | Exit the program             |

---

## ⚙️ Configuration

The **`rush.toml`** file is automatically created in the `.rss` directory on first start.

### Complete Default Configuration

```toml
[general]
max_messages = 100
# Typewriter-Effekt: 50ms = 20 Zeichen/Sekunde (empfohlen: 30-100ms)
typewriter_delay = 50
input_max_length = 100
max_history = 30
# Poll-Rate: 16ms = 60 FPS (empfohlen: 16-33ms, NICHT unter 16!)
poll_rate = 16
log_level = "info"

[theme]
input_text = "Black"
input_bg = "White"
cursor = "Black"
output_text = "DarkGray"
output_bg = "Black"

[prompt]
text = "/// "
color = "Black"

[language]
current = "en"

# =================================================================
# PERFORMANCE-HINWEISE:
# =================================================================
# poll_rate:
#   - 16ms = 60 FPS (EMPFOHLEN für flüssiges UI)
#   - 33ms = 30 FPS (akzeptabel für langsamere Systeme)
#   - 1-15ms = NICHT empfohlen (hohe CPU-Last!)
#   - 0ms = CRASH! (Tokio interval panic)
#
# typewriter_delay:
#   - 50ms = 20 Zeichen/Sekunde (gut lesbar)
#   - 30ms = 33 Zeichen/Sekunde (schnell)
#   - 100ms = 10 Zeichen/Sekunde (langsam)
#   - 0ms = Typewriter-Effekt deaktiviert
# =================================================================
```

### Performance Optimization

**Recommended Values:**

- `poll_rate = 16` (60 FPS - optimal)
- `poll_rate = 33` (30 FPS - good for slower systems)
- `typewriter_delay = 0` (disabled) or `30-100` (enabled)

**Automatic Bounds Checking:**

- Invalid values are automatically corrected on startup
- Corrected values are saved back to config
- Performance warnings for critical settings

### Colors (COLOR_MAP)

Supported:
`Black`, `White`, `Gray`, `DarkGray`, `Red`, `Green`, `Blue`, `Yellow`,
`Magenta`, `Cyan`, `LightRed`, `LightGreen`, `LightBlue`, `LightYellow`,
`LightMagenta`, `LightCyan`

**Smart Color Categories:**

- `error` → Red
- `warning` / `warn` → Yellow
- `info` → Green
- `debug` → Blue
- `lang` / `language` → Cyan
- `version` → LightBlue

i18n translations are automatically mapped to standard keys
(e.g., `"Sprache"`, `"Idioma"`, `"Язык"` → `lang` → Cyan).

---

## 🗂 Project Structure

```graphql
src/
├── core/        # Core logic (Config, Error, Prelude, Constants)
├── ui/          # Terminal UI (ScreenManager, TerminalManager, Widgets)
├── input/       # Input handling (Keyboard, EventHandler)
├── output/      # Logging, MessageManager, Color, Performance
├── commands/    # Modular commands (exit, lang, history, restart, log-level, performance)
├── setup/       # Auto-configuration (TOML setup)
└── i18n/        # Internationalization (German/English)
    └── langs/   # Language files (de.json, en.json)
```

---

## 🛠 Technical Details

- **Event loop:** Asynchronous (Tokio) → split into:
  - `handle_input_event`
  - `handle_tick_event`
  - `handle_resize_event`
- **Logging:**
  - Global `AppLogger` (intercepts all `log::*` calls)
  - Runtime log-level switching with config persistence
  - `LogMessage` stores level + text → color-coded output
- **Performance Monitoring:**
  - Real-time FPS calculation based on poll_rate
  - Typewriter speed analysis (chars/second)
  - Config value validation with automatic correction
  - Performance recommendations based on system capabilities
- **Internationalization:**
  - `get_marker_color` automatically maps translated markers to standard categories
  - Smart fallback for unknown display categories
  - Persistent language switching with config save
- **Restart:** Internal, without external process restart
- **Memory Management:** Bounded message buffers with automatic cleanup

---

## 🧪 Testing

```bash
cargo test
RUST_LOG=debug cargo test

# Test specific components
cargo test command_system_tests
cargo test performance
cargo test config
```

Available tests:
✔ Commands (including new log-level & performance)
✔ Event loop
✔ Config setup & bounds checking
✔ i18n translations (German/English)
✔ Performance monitoring
✔ Language switching

---

## 🎛 Advanced Usage

### Performance Monitoring

```bash
# Show detailed performance report
perf

# Output example:
📊 PERFORMANCE & CONFIG STATUS:
🎯 Poll Rate: 16ms (60.0 FPS) ✅ Optimal
⌨️ Typewriter: 50ms (20.0 chars/sec)
📈 Max Messages: 100
📜 Max History: 30
🎨 Log Level: INFO
📍 Config: rush.toml

💡 EMPFEHLUNGEN:
• poll_rate: 16ms (optimal) oder 33ms (gut)
• typewriter_delay: 0ms (aus) oder 30-100ms
• Für beste Performance: poll_rate >= 16ms
```

### Dynamic Log-Level Management

```bash
# Runtime log-level switching (saved to config)
log-level debug    # Switch to debug mode
log-level 1        # Switch to error-only
log-level info     # Back to default
```

### Language Switching

```bash
lang de            # Switch to German
lang en            # Switch to English
lang               # Show current language & available options
```

### **Library Integration Examples**

#### **Create Custom Commands**

```rust
use rush_sync_server::*;

// Define a custom command
#[derive(Debug)]
struct HelloCommand;

impl Command for HelloCommand {
    fn name(&self) -> &'static str { "hello" }
    fn description(&self) -> &'static str { "Say hello" }
    fn matches(&self, command: &str) -> bool { command == "hello" }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let name = args.first().unwrap_or(&"World");
        Ok(format!("Hello, {}!", name))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Add custom command to existing registry
    let mut handler = create_handler();
    handler.add_command(HelloCommand);

    let result = handler.handle_input("hello Rust");
    println!("{}", result.message); // "Hello, Rust!"

    Ok(())
}
```

#### **Use with Custom UI**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config
    let config = load_config().await?;

    // Create handler for command processing
    let handler = create_handler();

    // Simple CLI loop (instead of full TUI)
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let result = handler.handle_input(&input.trim());
        println!("{}", result.message);

        if result.should_exit {
            break;
        }
    }

    Ok(())
}
```

---

## 🗺 Roadmap

- [ ] Mouse support (scroll & selection)
- [ ] Split-screen & tabs
- [ ] Syntax highlighting
- [x] ~~Plugin system for custom commands~~ ✓ Implemented
- [x] ~~Live log-level configuration~~ ✓ Implemented
- [x] ~~Performance monitoring~~ ✓ Implemented
- [x] ~~Binary & Library distribution~~ ✓ Implemented (v0.2.3+)
- [ ] Live UI configuration changes
- [ ] Config hot-reload without restart
- [ ] Custom color themes
- [ ] Command aliases & macros

---

## 📊 **What's New in v0.2.3**

- ✅ **Binary distribution**: Install with `cargo install rush-sync-server`
- ✅ **Library API**: Use as dependency in your Rust projects
- ✅ **Convenience functions**: `run()`, `load_config()`, `create_handler()`
- ✅ **Public exports**: Access to `Config`, `CommandHandler`, `ScreenManager`
- ✅ **Custom command support**: Extend functionality with your own commands
- ✅ **Hybrid usage**: Use as standalone app OR integrate into your project

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

**Development Guidelines:**

- Follow performance recommendations (poll_rate >= 16ms)
- Add i18n support for new features (de.json + en.json)
- Include performance tests for new commands
- Update config bounds checking for new parameters
