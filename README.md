# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> 🛠 **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.2.6+`** for a stable release!

**Rush Sync Server** is a modern, modular terminal application written in **Rust**, featuring an interactive TUI, internationalized interface, color-coded logging, and flexible configuration.
Perfect for developers who need a **customizable, scriptable terminal UI**.

---

## 🆕 What's New in v0.2.6

- ✅ **Fixed PIPE Cursor Rendering** - PIPE cursor now renders its own symbol instead of using terminal cursor, enabling **full color support** from TOML themes
- ✅ **Reorganized Configuration** - `.rss/rush.toml` has been completely restructured and sorted for better readability and organization
- ✅ **Zero Warnings & Errors** - All `cargo clippy` and `cargo check` warnings have been resolved - completely clean codebase!
- ✅ **Improved Cursor System** - Complete separation between terminal cursor and application cursor rendering for better control
- ✅ **Enhanced Theme Support** - All cursor colors and styles now work perfectly via TOML configuration

### Previous Features (v0.2.5)

- ✅ **Live Theme Switching** at runtime (without restart)
- ✅ **Advanced Cursor System** (PIPE, BLOCK, UNDERSCORE, DEFAULT)
- ✅ **Unified Cursor Architecture** (input/output cursor with blinking, positioning, color)
- ✅ **Centralized Viewport** for smooth layout handling and robust scrolling
- ✅ **Improved Restart Logic** with UI reinitialization and state restore
- ✅ **Theme-defined cursor styles and colors** via TOML
- ✅ **Full i18n coverage** for logs, errors, and commands

---

## 🚀 Installation & Usage

### 📦 **As Binary (End Users) - Version 0.2.6+**

```bash
# Install directly from crates.io
cargo install rush-sync-server

# Run the application
rush-sync
```

### 📚 **As Library (Developers) - Version 0.2.6+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.2.6"
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
- **Advanced cursor system** with full color support via TOML themes
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
- **Zero warnings codebase** - completely clean `cargo clippy` and `cargo check`

---

## 💻 Available Commands

| Command                | Description                                     | Examples                         |
| ---------------------- | ----------------------------------------------- | -------------------------------- |
| `version` / `ver`      | Show application version                        | `version`                        |
| `lang` / `language`    | Switch language (EN/DE) with config persistence | `lang de`, `lang en`             |
| `theme`                | Change themes live (from TOML configuration)    | `theme dark`, `theme light`      |
| `clear` / `cls`        | Clear all messages                              | `clear`                          |
| `exit` / `q`           | Exit with confirmation                          | `exit`                           |
| `restart`              | Internal restart (reloads config)               | `restart`, `restart --force`     |
| `history -c`           | Clear input history                             | `history -c`                     |
| `log-level`            | Change log level (runtime + persistent)         | `log-level 3`, `log-level debug` |
| `perf` / `performance` | Show performance & config status                | `perf`, `performance`            |

### Theme Command Details

```bash
theme                # Show available themes from TOML
theme dark           # Switch to dark theme
theme light          # Switch to light theme
theme blue           # Switch to blue theme
theme green          # Switch to green theme
theme preview <name> # Preview theme without switching
theme -h             # Show help
```

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

### Complete Default Configuration (v0.2.6 - Reorganized & Sorted)

```toml
[general]
max_messages = 100
typewriter_delay = 5
input_max_length = 100
max_history = 30
poll_rate = 16
log_level = "info"
current_theme = "dark"

[language]
current = "en"

[theme.dark]
output_bg = "Black"
output_text = "White"
output_cursor = "PIPE"
output_cursor_color = "White"
input_bg = "White"
input_text = "Black"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "Black"

[theme.light]
output_bg = "White"
output_text = "Black"
output_cursor = "PIPE"
output_cursor_color = "Black"
input_bg = "Black"
input_text = "White"
input_cursor_prefix = "/// "
input_cursor = "PIPE"
input_cursor_color = "White"

[theme.green]
output_bg = "Black"
output_text = "Green"
output_cursor = "BLOCK"
output_cursor_color = "Green"
input_bg = "LightGreen"
input_text = "Black"
input_cursor_prefix = "$ "
input_cursor = "BLOCK"
input_cursor_color = "Black"

[theme.blue]
output_bg = "White"
output_text = "LightBlue"
output_cursor = "UNDERSCORE"
output_cursor_color = "Blue"
input_bg = "Blue"
input_text = "White"
input_cursor_prefix = "> "
input_cursor = "UNDERSCORE"
input_cursor_color = "White"
```

### 🎨 Theme Configuration Details

**New in v0.2.6:**

- **Perfect cursor color support** - All cursor colors now work correctly via TOML
- **Clean theme structure** - Organized output-first, then input configuration
- **Multiple cursor types** - PIPE, BLOCK, UNDERSCORE all fully supported with colors

**Theme Structure:**

```toml
[theme.your_theme_name]
# Output area configuration
output_bg = "Background color for output area"
output_text = "Text color for output messages"
output_cursor = "PIPE|BLOCK|UNDERSCORE"
output_cursor_color = "Color for output cursor"

# Input area configuration
input_bg = "Background color for input area"
input_text = "Text color for input"
input_cursor_prefix = "Prompt text (e.g., '/// ')"
input_cursor = "PIPE|BLOCK|UNDERSCORE"
input_cursor_color = "Color for input cursor"
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
├── commands/    # Modular commands (exit, lang, history, restart, log-level, performance, theme)
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
- **Cursor System (v0.2.6):**
  - Complete separation between terminal cursor and application cursor
  - PIPE cursor renders its own symbol with full color support
  - BLOCK and UNDERSCORE cursors with configurable colors
  - Blinking animation and positioning independent of terminal
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
- **Code Quality:** Zero warnings from `cargo clippy` and `cargo check`

---

## 🧪 Testing

```bash
cargo test
RUST_LOG=debug cargo test

# Test specific components
cargo test command_system_tests
cargo test performance
cargo test config
cargo test theme_system

# Code quality checks (all pass with zero warnings!)
cargo clippy
cargo check
```

Available tests:
✔ Commands (including theme, log-level & performance)
✔ Event loop
✔ Config setup & bounds checking
✔ i18n translations (German/English)
✔ Performance monitoring
✔ Language switching
✔ Theme system
✔ Cursor rendering

---

## 🎛 Advanced Usage

### Live Theme Switching

```bash
# Show available themes from TOML
theme

# Switch themes instantly (no restart required)
theme dark           # Professional dark theme
theme light          # Clean light theme
theme green          # Terminal green theme with BLOCK cursor
theme blue           # Modern blue theme with UNDERSCORE cursor

# Preview before switching
theme preview green  # See theme details without changing
```

### Performance Monitoring

```bash
# Show detailed performance report
perf

# Output example:
📊 COMPREHENSIVE PERFORMANCE REPORT
==================================================

🎯 System Performance
   • Poll Rate: 16ms (60.0 FPS) ✅
   • Typewriter Speed: 5ms (200.0 chars/sec)

💾 Memory Usage
   • Total Estimated: 0.05 MB
   • Message Buffer: 0.01 MB
   • History Buffer: 0.00 MB
   • i18n Cache: 0.50 MB

💡 Recommendations
   • ✅ All settings optimally configured

🔧 Related Commands
   • log-level debug - Enable debug logging
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
- [x] ~~Live theme switching~~ ✓ Implemented (v0.2.5+)
- [x] ~~Advanced cursor system with colors~~ ✓ Implemented (v0.2.6+)
- [ ] Config hot-reload without restart
- [ ] Custom color themes editor
- [ ] Command aliases & macros
- [ ] Plugin marketplace

---

## 📊 **Version History**

### **v0.2.6 (Latest) - Cursor & Quality Update**

- ✅ **Fixed PIPE cursor rendering** with full color support from TOML
- ✅ **Reorganized `.rss/rush.toml`** configuration with sorted structure
- ✅ **Zero warnings codebase** - all `cargo clippy` and `cargo check` issues resolved
- ✅ **Enhanced cursor system** with complete terminal separation

### **v0.2.5 - Live Theme Update**

- ✅ **Live theme switching** at runtime without restart
- ✅ **Advanced cursor architecture** with unified input/output handling
- ✅ **Centralized viewport** with smooth scrolling

### **v0.2.3 - Library Release**

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
- Ensure zero warnings with `cargo clippy`
- Test theme configurations thoroughly

---

## 🏆 Code Quality

**Rush Sync Server v0.2.6** maintains the highest code quality standards:

- ✅ **Zero Clippy Warnings** - Complete compliance with Rust best practices
- ✅ **Zero Cargo Check Errors** - All code compiles cleanly
- ✅ **100% Functional** - All features work as documented
- ✅ **Comprehensive Tests** - Full test coverage for critical components
- ✅ **Clean Architecture** - Well-structured, modular codebase
- ✅ **Memory Safe** - No unsafe code, bounded buffers, proper cleanup
