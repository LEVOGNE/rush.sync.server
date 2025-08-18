# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> 🛠 **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.2.9+`** for a stable release!

**Rush Sync Server** is an ambitious project to create a **professional web server orchestration platform** written in Rust. Currently in Phase 0 (Foundation), the project features a robust terminal UI with internationalization, theming, and command system - laying the groundwork for the upcoming server management capabilities.

---

## 🎯 Project Vision

Rush Sync Server is being developed in phases:

- **Phase 0** (Current): Terminal UI foundation with command system ✅
- **Phase 1**: Dynamic Actix-Web server management
- **Phase 2**: Dev/Prod modes with hot-reloading
- **Phase 3**: Redis integration & secure communication
- **Phase 4**: Centralized logging & automation

---

## 🆕 What's New in v0.2.9

### **🎉 Major Foundation Improvements**

- **🖥️ Complete Screen Management Overhaul** with robust viewport handling
- **📜 Advanced Scroll System** with smooth navigation and position preservation
- **🎨 Terminal Cursor Integration** - Real terminal cursor synchronized with text rendering
- **🔄 Enhanced Live Theme Updates** with complete UI state preservation
- **🛡️ Bulletproof Input State Management** with backup/restore functionality
- **📱 Multi-Terminal Platform Support** with intelligent cursor color handling
- **⚡ Optimized Rendering Pipeline** - 2-layer rendering (text + terminal cursor)
- **🧹 Complete Code Cleanup** - Removed debug commands, focused on core functionality

### **🖱️ Advanced Cursor System**

- **🎯 Unified Cursor API** - One system for both input and output cursors
- **🎨 Multi-Platform Terminal Detection** - macOS Terminal, iTerm2, VSCode, tmux auto-detection
- **🌈 Dynamic Cursor Colors** - Real terminal cursor color changes (yellow, blue, green, etc.)
- **⚡ Terminal Cursor Synchronization** - Separate layer for true terminal cursor positioning
- **🔄 Live Cursor Updates** - Cursor type and color changes without restart

### **📺 Screen & Viewport Enhancements**

- **📐 Robust Layout Calculation** with panic-safe dimension handling
- **📜 Smart Scroll Management** - Auto-scroll preservation and manual scroll detection
- **🔄 Content Height Tracking** with intelligent cache management
- **📊 Advanced Viewport Events** - Unified event system for screen changes
- **🎯 Precise Visible Range Calculation** for optimal message rendering
- **🚨 Emergency Layout Fallbacks** for edge cases and small terminals

### **🎨 Theme System Improvements**

- **🔄 Complete UI State Preservation** during theme changes
- **🎯 Input Cursor Configuration** - Full control over prompt style, color, and cursor type
- **🌈 Output Cursor Configuration** - Configurable typewriter cursor appearance
- **📝 Improved Theme Preview** with detailed cursor information
- **🔧 Enhanced Theme Debugging** with comprehensive configuration display

### **⌨️ Input System Enhancements**

- **💾 Robust State Backup/Restore** - History and cursor position preservation
- **📋 Enhanced Clipboard Integration** - Cross-platform copy/paste with text cleaning
- **🛡️ Advanced Input Validation** - Better handling of special characters and Unicode
- **🔄 Improved History Management** - Persistent file-based history with proper loading
- **⚡ Optimized Input Processing** - Faster response times and better event handling

### **🌍 Internationalization Improvements**

- **🇩🇪 Complete German Translation Coverage** for all new features
- **🎨 Enhanced Color Category Mapping** with i18n support
- **📝 Localized Error Messages** for all new components
- **🔄 Runtime Translation Updates** with cache management

---

## 🚀 Installation & Usage

### 📦 **As Binary - Version 0.2.9+**

```bash
# Install from crates.io
cargo install rush-sync-server

# Run the terminal UI (current functionality)
rush-sync
```

### 📚 **As Library - Version 0.2.9+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.2.9"
tokio = { version = "1.36", features = ["full"] }
```

#### **Quick Start Examples:**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Run with default configuration
    run().await?;
    Ok(())
}
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Custom configuration
    let mut config = load_config().await?;
    config.poll_rate = std::time::Duration::from_millis(8); // 125 FPS
    config.typewriter_delay = std::time::Duration::from_millis(1); // Ultra-fast

    // Run with custom settings
    run_with_config(config).await?;
    Ok(())
}
```

### 🛠 **From Source**

```bash
git clone https://github.com/LEVOGNE/rush.sync.server
cd rush.sync.server
cargo build --release
cargo run --release
```

---

## ✅ Current Features (Phase 0 - Foundation Complete)

### **🏗️ Core Foundation**

- **Interactive Terminal UI** with asynchronous event loop (Tokio)
- **Modular Command System** with extensible architecture
- **Advanced Error Handling** with graceful recovery
- **Zero Warnings Codebase** (cargo clippy clean)
- **Memory-Safe Operations** with proper resource management

### **📊 Logging & Persistence**

- **Color-coded Logging** with levels (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`)
- **Runtime Log-level Switching** with persistent config save
- **📝 Message Logging** to `.rss/rush.logs` with timestamps
- **📚 Command History** in `.rss/rush.history` with file persistence
- **Auto-scroll & Message History** with smart viewport management

### **🎨 Advanced UI System**

- **🖱️ Intelligent Dual-Cursor System** - Text rendering + real terminal cursor
- **🎯 Multi-Platform Terminal Detection** (macOS Terminal, iTerm2, VSCode, tmux)
- **🌈 Dynamic Terminal Cursor Colors** with real-time color changes
- **Live Theme Switching** without restart (TOML-based)
- **Typewriter Effect** with configurable speed and cursor
- **Unicode Support** (grapheme-based text handling)
- **🎯 Advanced Viewport** with smooth scrolling and position preservation
- **📱 Responsive Layout System** with panic-safe dimension handling

### **⌨️ Input & Navigation**

- **Full Keyboard Support** - Shift + symbols, umlauts, Unicode & emoji
- **Platform Shortcuts** - Cmd (macOS) / Ctrl (Win/Linux) navigation
- **Input History Navigation** with arrow keys and persistent storage
- **📋 Enhanced Copy/Paste Integration** with text cleaning and validation
- **🛡️ Smart Input Validation** with length limits and sanitization
- **💾 State Backup/Restore** - Complete input state preservation

### **📺 Screen & Viewport Management**

- **📐 Robust Layout Calculation** with emergency fallbacks
- **📜 Advanced Scroll System** with auto-scroll detection
- **🎯 Precise Content Tracking** with intelligent cache management
- **🔄 Unified Event System** for viewport changes
- **🚨 Error Recovery** for layout failures and edge cases
- **📊 Performance-Optimized Rendering** with 2-layer architecture

### **🌍 Internationalization**

- **Runtime Language Switching** (German/English)
- **Color-coded Command Categories** with i18n support
- **Localized Error Messages** and help texts
- **Extensible Translation System** for future languages

### **⚙️ Configuration & Themes**

- **Smart Config Validation** with auto-correction
- **TOML-based Theme System** with live updates
- **Internal Restart** without process termination
- **Persistent Settings** with automatic backup

---

## 💻 Available Commands

| Command             | Description              | Examples                     |
| ------------------- | ------------------------ | ---------------------------- |
| `version` / `ver`   | Show application version | `version`                    |
| `lang` / `language` | Switch language (EN/DE)  | `lang de`, `lang en`         |
| `theme`             | Change themes live       | `theme dark`, `theme light`  |
| `clear` / `cls`     | Clear all messages       | `clear`                      |
| `exit` / `q`        | Exit with confirmation   | `exit`                       |
| `restart`           | Internal restart         | `restart`, `restart --force` |
| `history -c`        | Clear input history      | `history -c`                 |
| `log-level`         | Change log level         | `log-level debug`            |

### 🎨 Theme Commands

```bash
theme                # Show available themes from TOML
theme dark           # Switch to dark theme (live update)
theme preview <name> # Preview theme without switching
theme debug <name>   # Show detailed theme configuration including cursor settings
theme -h             # Show comprehensive help with cursor options
```

### 📊 Log-Level Commands

```bash
log-level           # Show current level and help
log-level 3         # Set to INFO (1=ERROR, 2=WARN, 3=INFO, 4=DEBUG, 5=TRACE)
log-level DEBUG     # Set by name (case-insensitive)
log-level -h        # Show detailed help
```

### 📚 History Commands

```bash
history             # Show help and current status
history -c          # Clear command history
↑ / ↓               # Navigate through history
```

---

## ⌨️ Enhanced Keyboard Shortcuts

### **🔤 Text Navigation**

| Key            | Function            |
| -------------- | ------------------- |
| `← / →`        | Move cursor in text |
| `Home / End`   | Jump to start/end   |
| `Cmd/Ctrl + A` | Jump to start       |
| `Cmd/Ctrl + E` | Jump to end         |

### **📝 Text Editing**

| Key            | Function             |
| -------------- | -------------------- |
| `Backspace`    | Delete previous char |
| `Delete`       | Delete next char     |
| `Cmd/Ctrl + U` | Clear entire line    |
| `Cmd/Ctrl + C` | Copy current input   |
| `Cmd/Ctrl + V` | Paste from clipboard |
| `Cmd/Ctrl + X` | Cut current input    |

### **📚 History & Navigation**

| Key              | Function               |
| ---------------- | ---------------------- |
| `↑ / ↓`          | Navigate input history |
| `Shift + ↑ / ↓`  | Scroll messages        |
| `Page Up / Down` | Page scroll            |

### **🎛️ Application Control**

| Key        | Function         |
| ---------- | ---------------- |
| `Enter`    | Submit command   |
| `ESC` (2x) | Exit application |

---

## ⚙️ Configuration System

### **📁 File Locations**

- **Config**: `.rss/rush.toml` (auto-created)
- **History**: `.rss/rush.history` (persistent command history)
- **Logs**: `.rss/rush.logs` (timestamped message log)

### **🛠 Configuration File**

```toml
[general]
max_messages = 1000         # Message buffer size
typewriter_delay = 5        # Typewriter effect speed (0 = disabled)
input_max_length = 100      # Maximum input length
max_history = 30            # Command history entries
poll_rate = 16              # UI refresh rate (16ms = 62.5 FPS)
log_level = "info"          # Log level (error/warn/info/debug/trace)
current_theme = "dark"      # Active theme name

[language]
current = "en"              # Language (en/de)

# Built-in themes with advanced cursor configuration
[theme.dark]
output_bg = "Black"
output_text = "White"
output_cursor = "PIPE"           # PIPE, BLOCK, UNDERSCORE
output_cursor_color = "White"    # Terminal cursor color for typewriter
input_bg = "White"
input_text = "Black"
input_cursor_prefix = "/// "     # Prompt text
input_cursor = "PIPE"            # Input cursor type
input_cursor_color = "Black"     # Input cursor color (real terminal cursor)

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

# 🆕 Advanced theme with yellow cursor
[theme.yellow]
output_bg = "Black"
output_text = "Yellow"
output_cursor = "PIPE"
output_cursor_color = "Yellow"
input_bg = "DarkGray"
input_text = "Yellow"
input_cursor_prefix = "⚡ "
input_cursor = "PIPE"
input_cursor_color = "Yellow"     # Real terminal cursor will be yellow!
```

### **🎨 Supported Colors**

**Standard Colors:**
`Black`, `White`, `Gray`, `DarkGray`, `Red`, `Green`, `Blue`, `Yellow`, `Magenta`, `Cyan`

**Light Variants:**
`LightRed`, `LightGreen`, `LightBlue`, `LightYellow`, `LightMagenta`, `LightCyan`

### **🖱️ Cursor Types**

- **PIPE** (`|`) - Vertical line cursor
- **BLOCK** (`█`) - Block cursor (inverts character)
- **UNDERSCORE** (`_`) - Underscore cursor

### **🌈 Terminal Cursor Colors**

**New in v0.2.9:** Real terminal cursor color changes based on theme configuration!

- **macOS Terminal.app** - Standard OSC sequences
- **iTerm2** - Native color support with multiple fallback sequences
- **VSCode Terminal** - Standard compatibility mode
- **tmux** - Proper tmux-wrapped sequences
- **Generic Terminals** - Universal fallback sequences

---

## 🔧 Advanced Features

### **🛡️ Terminal Compatibility**

- **🍎 macOS Terminal.app** - Optimized sequences with RGB color support
- **⚡ iTerm2** - Native color support with multiple sequence types
- **💻 VSCode Terminal** - Standard compatibility with fallback handling
- **🔄 tmux Sessions** - Proper tmux-wrapped escape sequence handling
- **🌐 Generic Terminals** - Universal fallback support with error recovery

### **🖱️ Advanced Cursor System**

```bash
# Real-time cursor changes in terminal
theme blue    # Terminal cursor becomes blue
theme yellow  # Terminal cursor becomes yellow
theme green   # Terminal cursor becomes green

# Cursor debugging
theme debug dark    # Shows detailed cursor configuration
```

### **📺 Viewport Management**

- **📐 Panic-Safe Layout Calculation** - Emergency fallbacks for edge cases
- **📜 Smart Auto-Scroll Detection** - Preserves manual scroll position
- **🎯 Precise Content Tracking** - Optimized message rendering
- **🔄 Event-Driven Updates** - Unified system for all viewport changes
- **📊 Performance-Optimized Rendering** - 2-layer architecture (text + cursor)

### **📊 Intelligent Logging**

```bash
# Message logs with timestamps
[2024-01-15 14:30:25] System started
[2024-01-15 14:30:26] Theme changed to: DARK
[2024-01-15 14:30:30] Language switched to: DE
[2024-01-15 14:30:35] Terminal cursor color changed to: Yellow

# Automatic log rotation and size management
```

### **🔄 Error Recovery**

- **Graceful Panic Handling** with complete terminal cleanup
- **Poisoned Mutex Recovery** for thread safety
- **Config Validation** with automatic correction
- **File System Error Handling** with fallbacks
- **Layout Failure Recovery** with emergency layouts
- **🆕 Terminal State Recovery** - Cursor and color reset on exit

---

## 🗺 Development Roadmap

### **Phase 1: Server Management (Next)**

- [ ] CLI commands: `create`, `start`, `stop`, `delete`, `status`, `logs`
- [ ] Dynamic Actix-Web server spawning
- [ ] Hash-based server isolation
- [ ] Ghost mode (background execution)
- [ ] JSON/SQLite server registry

### **Phase 2: Dev/Prod & Versioning**

- [ ] Dev mode with hot-reloading
- [ ] Prod mode with TLS
- [ ] Automatic versioning (v1, v2, ...)
- [ ] File watcher with `notify`
- [ ] SCSS compilation

### **Phase 3: Communication & Security**

- [ ] Redis Pub/Sub integration
- [ ] TLS/HTTPS with `rustls`
- [ ] Session caching
- [ ] Inter-server communication

### **Phase 4: Logging & Automation**

- [ ] Centralized logging dashboard
- [ ] Automated setup scripts
- [ ] WebSocket support
- [ ] Integration tests

### **Future Considerations**

- [ ] Load balancing
- [ ] Docker integration
- [ ] Kubernetes support
- [ ] Web-based monitoring dashboard

---

## 🗂 Project Structure

### **Current Structure (v0.2.9)**

```bash
src/
├── core/           # Core logic & configuration
│   ├── config.rs   # TOML config with enhanced theme system
│   ├── error.rs    # Comprehensive error handling
│   ├── constants.rs # Application constants
│   └── prelude.rs  # Common imports with enhanced traits
├── ui/             # Advanced terminal UI
│   ├── screen.rs   # 🆕 Complete screen management overhaul
│   ├── terminal.rs # Enhanced terminal initialization
│   ├── cursor.rs   # 🆕 Unified cursor system (input/output)
│   ├── viewport.rs # 🆕 Advanced scroll & layout management
│   ├── widget.rs   # Enhanced UI widget traits
│   └── color.rs    # Color system with i18n
├── input/          # Enhanced input handling system
│   ├── keyboard.rs # 🆕 Improved keyboard with better filtering
│   ├── state.rs    # 🆕 Enhanced input state with backup/restore
│   └── mod.rs      # Optimized event loop
├── output/         # Enhanced display & logging
│   └── display.rs  # 🆕 Advanced message display with viewport integration
├── commands/       # Streamlined command system
│   ├── clear/      # Clear command
│   ├── exit/       # Exit with confirmation
│   ├── history/    # Enhanced history management
│   ├── lang/       # Language switching
│   ├── log_level/  # Log level control
│   ├── restart/    # Internal restart
│   ├── theme/      # 🆕 Enhanced live theme system
│   ├── version/    # Version display
│   ├── command.rs  # Command trait
│   ├── handler.rs  # Command processing
│   └── registry.rs # Command registry
├── setup/          # Auto-configuration
│   └── setup_toml.rs # 🆕 Enhanced config with sorted themes
└── i18n/           # Enhanced internationalization
    ├── mod.rs      # Translation engine
    └── langs/      # Language files
        ├── en.json # 🆕 Extended English translations
        └── de.json # 🆕 Extended German translations
```

### **Planned Structure (Phase 1+)**

```bash
src/
├── cli/            # Server management CLI
├── server/         # Actix-Web management
├── db/             # Redis & PostgreSQL
├── versioning/     # Version control
└── websocket/      # Real-time communication
```

---

## 🧪 Testing & Quality Assurance

### **🔍 Code Quality Checks**

```bash
# Zero warnings guarantee
cargo clippy --all-targets --all-features
cargo check --all-targets
cargo test --all-features

# Specific component tests
cargo test command_system_tests
cargo test config_validation
cargo test theme_system
cargo test i18n_system
cargo test input_handling
cargo test viewport_management
cargo test cursor_system
```

### **🛡️ Security Testing**

```bash
# Input sanitization tests
cargo test escape_sequence_filtering
cargo test input_validation
cargo test file_operations

# Error recovery tests
cargo test panic_recovery
cargo test mutex_poisoning
cargo test config_corruption
cargo test layout_failure_recovery
```

### **🎨 UI System Tests**

```bash
# Viewport and rendering tests
cargo test viewport_calculations
cargo test scroll_management
cargo test cursor_positioning
cargo test theme_switching
cargo test terminal_compatibility
```

---

## 📊 Version History

### **v0.2.9 (Current) - Screen & Cursor System Complete**

**🎉 Major Features:**

- 🖥️ Complete screen management overhaul with robust viewport handling
- 📜 Advanced scroll system with smooth navigation and auto-scroll detection
- 🎨 Terminal cursor integration - Real terminal cursor synchronized with text
- 🔄 Enhanced live theme updates with complete UI state preservation
- 🛡️ Bulletproof input state management with backup/restore functionality

**🛠️ Technical Improvements:**

- ⚡ 2-layer rendering architecture (text + terminal cursor)
- 🎯 Unified cursor API for both input and output cursors
- 📱 Multi-platform terminal detection with cursor color support
- 📐 Panic-safe layout calculations with emergency fallbacks
- 🔄 Enhanced viewport event system with unified management

**🎨 UI/UX Enhancements:**

- 🌈 Dynamic terminal cursor colors (yellow, blue, green, etc.)
- 🎯 Precise cursor positioning with platform-specific handling
- 📊 Performance-optimized message rendering
- 🚨 Improved error recovery for layout failures
- 📱 Better responsive design for small terminals

### **v0.2.8 - Foundation Complete**

**🎉 Major Features:**

- 📝 Persistent message logging to `.rss/rush.logs`
- 📚 Persistent command history in `.rss/rush.history`
- 🛡️ Advanced terminal compatibility with escape sequence detection
- 🔧 Enhanced error handling throughout codebase
- 🧹 Code architecture cleanup (removed performance module)

**🛠️ Technical Improvements:**

- ⚡ Optimized event processing with input filtering
- 🔒 Advanced input sanitization and validation
- 💾 Safe file operations with proper error handling
- 🔄 Improved terminal cleanup on panic/exit
- 🎯 Preparation for Phase 1 server management

### **v0.2.7 - Input System Complete**

- ✅ Full keyboard input support (Shift + symbols, umlauts)
- ✅ Platform-specific shortcuts (Cmd/Ctrl)
- ✅ Terminal reset improvements
- ✅ Copy/paste integration

### **v0.2.6 - UI Polish**

- ✅ Fixed PIPE cursor rendering issues
- ✅ Zero warnings codebase achievement
- ✅ Enhanced viewport management

### **v0.2.5 - Theme System**

- ✅ Live theme switching without restart
- ✅ Advanced cursor system with TOML configuration
- ✅ Multi-cursor type support (PIPE, BLOCK, UNDERSCORE)

### **v0.2.3 - Public Release**

- ✅ Binary & library distribution
- ✅ Public API for developers
- ✅ Comprehensive documentation

---

## 🏆 Code Quality Metrics

**Rush Sync Server v0.2.9** maintains exceptional standards:

- ✅ **Zero Clippy Warnings** (all lints passing)
- ✅ **Zero Cargo Check Errors** (clean compilation)
- ✅ **Memory Safe** (Rust guarantees + manual verification)
- ✅ **Thread Safe** (proper async/sync boundaries)
- ✅ **Comprehensive Error Handling** (Result types throughout)
- ✅ **Clean Architecture** (modular design patterns)
- ✅ **Extensive Testing** (unit + integration tests)
- ✅ **Documentation Coverage** (all public APIs documented)
- ✅ **🆕 Panic-Safe Operations** (emergency fallbacks for all critical paths)
- ✅ **🆕 Cross-Platform Compatibility** (tested on macOS, Linux, Windows)

---

## 📜 License

### **Dual-Licensing Model**

1. **Community License (GPLv3)** — Free for private and non-commercial use
2. **Commercial License** — Required for commercial applications

**For commercial licensing inquiries:**
📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)

---

## 🤝 Contributing

### **🎯 Areas Looking for Contributors:**

**Phase 1 Development:**

- Web server management with Actix-Web
- Redis integration and caching
- Database design (SQLite/PostgreSQL)
- DevOps automation tools

**Core Improvements:**

- Additional language translations
- Theme design and UX improvements
- Performance optimizations
- Cross-platform testing
- Terminal compatibility testing

### **📋 Development Guidelines:**

1. **Code Quality:**

   - Ensure zero warnings with `cargo clippy`
   - Add comprehensive tests for new features
   - Maintain memory safety and thread safety
   - Include panic-safe error handling

2. **Internationalization:**

   - Add i18n support for all new user-facing text
   - Update both `en.json` and `de.json` files
   - Test language switching functionality

3. **Configuration:**

   - Update config validation for new parameters
   - Provide sensible defaults and auto-correction
   - Test all theme configurations including cursor settings

4. **Documentation:**
   - Update README.md for new features
   - Add inline documentation for public APIs
   - Include usage examples
   - Document terminal compatibility notes

---

## 📞 Contact & Support

- **Primary Contact:** 📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)
- **GitHub Repository:** [LEVOGNE/rush.sync.server](https://github.com/LEVOGNE/rush.sync.server)
- **Issues & Bug Reports:** [GitHub Issues](https://github.com/LEVOGNE/rush.sync.server/issues)
- **Feature Requests:** [GitHub Discussions](https://github.com/LEVOGNE/rush.sync.server/discussions)

---

_Rush Sync Server v0.2.9 - Advanced UI foundation complete. Screen management, cursor system, and viewport handling perfected. Ready for Phase 1 server orchestration._
