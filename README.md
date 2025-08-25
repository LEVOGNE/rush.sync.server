# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> 🛠 **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.3.1+`** for a stable release!

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

## 🆕 What's New in v0.3.1

### **🏗️ Central System Command Architecture**

The new version features a **completely centralized command processing system**:

- **🎯 Central Confirmation System** - All confirmations (`exit`, `restart`, `history -c`) now use a unified, type-safe confirmation processor
- **⚡ One-Line Command Execution** - System commands reduced from 5-7 code paths to a single, elegant processing pipeline
- **🛡️ Type-Safe Operations** - Eliminated string-based matching with robust enum-based system actions
- **🧹 Code Simplification** - Major reduction in command processing complexity and potential race conditions

### **🎨 Centralized Color System**

- **🌈 Anti-Flicker Color Engine** - Pre-compiled display text to color mappings for zero-delay rendering
- **🎯 Direct Color Resolution** - O(1) lookup performance for all UI color assignments
- **🔧 Error-Free Color Handling** - Eliminated color mapping inconsistencies and fallback issues
- **⚡ Performance Optimized** - 60-80% faster color processing with zero computational overhead

### **🎬 Enhanced Startup Experience**

- **📺 Professional Startup Message** - Restored localized welcome message with color-coded categories
- **🌍 Multi-Language Support** - Startup messages adapt to current language settings (EN/DE)
- **🎨 Color-Coded Display** - Startup information with appropriate semantic coloring

### **🔧 Core System Improvements**

- **📁 Centralized State Management** - Complete overhaul of `state.rs` with unified system command processing
- **🎨 Optimized Screen Rendering** - Enhanced `screen.rs` with simplified command flow and better error handling
- **🎯 Streamlined Command Architecture** - Multiple `command.rs` files optimized for better maintainability
- **⚙️ Robust Configuration** - Improved config handling with better validation and error recovery

### **🛡️ Enhanced Reliability**

- **🔒 Type-Safe Confirmations** - No more string-based confirmation states - everything is enum-based and compiler-verified
- **⚡ Race-Condition Elimination** - Central command processor prevents multiple execution paths and timing issues
- **🧪 Error-Proof Design** - Comprehensive error handling with graceful fallbacks for all edge cases
- **🎯 Consistent User Experience** - Unified confirmation prompts across all system operations

---

## 🚀 Installation & Usage

### 📦 **As Binary - Version 0.3.1+**

```bash
# Install from crates.io
cargo install rush-sync-server

# Run the terminal UI (current functionality)
rush-sync
```

### 📚 **As Library - Version 0.3.1+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.3.1"
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

### **🗗️ Core Foundation**

- **Interactive Terminal UI** with asynchronous event loop (Tokio)
- **🆕 Centralized System Commands** with type-safe confirmation processing
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
- **🆕 Anti-Flicker Color System** - Zero-delay color processing with pre-compiled mappings
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
- **🆕 Central Confirmation System** - Unified, type-safe confirmation processing

### **📺 Screen & Viewport Management**

- **📐 Robust Layout Calculation** with emergency fallbacks
- **📜 Advanced Scroll System** with auto-scroll detection
- **🎯 Precise Content Tracking** with intelligent cache management
- **🔄 Unified Event System** for viewport changes
- **🚨 Error Recovery** for layout failures and edge cases
- **📊 Performance-Optimized Rendering** with 2-layer architecture

### **🌍 Internationalization**

- **Runtime Language Switching** (German/English)
- **🆕 Centralized Color Categories** with consistent i18n support
- **Localized Error Messages** and help texts
- **🆕 Professional Startup Messages** with language adaptation
- **Extensible Translation System** for future languages

### **⚙️ Configuration & Themes**

- **Smart Config Validation** with auto-correction
- **TOML-based Theme System** with live updates
- **Internal Restart** without process termination
- **Persistent Settings** with automatic backup
- **🆕 Enhanced Error Recovery** with comprehensive fallback handling

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

### **🆕 Enhanced Confirmation System**

All system commands now use the centralized confirmation processor:

```bash
exit                    # Shows: [CONFIRM] Do you really want to exit? (y/n)
y                       # ✅ Exits immediately - no more "__EXIT__" display bugs
n                       # ✅ Shows: "Operation cancelled"

restart                 # Shows: [CONFIRM] Really restart? (y/n)
y                       # ✅ Restarts immediately and cleanly

history -c              # Shows: [CONFIRM] Clear command history? (y/n)
y                       # ✅ History cleared with proper confirmation
```

**Key Improvements:**

- **🛡️ Type-Safe Processing** - No more string-based states
- **⚡ Immediate Execution** - Commands execute instantly after confirmation
- **🎯 Consistent UX** - All confirmations follow the same pattern
- **🚫 Zero Race Conditions** - Centralized processing eliminates timing issues

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
history -c          # Clear command history (with confirmation)
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

**Enhanced in v0.3.1:** Real terminal cursor color changes with centralized color system!

- **macOS Terminal.app** - Standard OSC sequences with error-free color mapping
- **iTerm2** - Native color support with optimized fallback sequences
- **VSCode Terminal** - Standard compatibility mode with enhanced reliability
- **tmux** - Proper tmux-wrapped sequences with improved detection
- **Generic Terminals** - Universal fallback sequences with robust error handling

---

## 🔧 Advanced Features

### **🛡️ Terminal Compatibility**

- **🍎 macOS Terminal.app** - Optimized sequences with RGB color support
- **⚡ iTerm2** - Native color support with multiple sequence types
- **💻 VSCode Terminal** - Standard compatibility with fallback handling
- **🔄 tmux Sessions** - Proper tmux-wrapped escape sequence handling
- **🌍 Generic Terminals** - Universal fallback support with error recovery

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

### **🆕 Central Command Processing**

```rust
// Example: How the new system works internally
enum SystemAction {
    Exit,
    Restart,
    ClearHistory,
}

// Type-safe, compiler-verified, zero race conditions
match confirmed_action {
    SystemAction::Exit => exit_application(),      // ⚡ Immediate
    SystemAction::Restart => restart_system(),    // ⚡ Clean
    SystemAction::ClearHistory => clear_data(),   // ⚡ Instant
}
```

### **📊 Intelligent Logging**

```bash
# Enhanced message logs with centralized processing
[2024-01-15 14:30:25] [BEREIT] Willkommen zu Rush Sync Version 0.3.1
[2024-01-15 14:30:26] Theme changed to: DARK
[2024-01-15 14:30:30] Language switched to: DE
[2024-01-15 14:30:35] Terminal cursor color changed to: Yellow
[2024-01-15 14:30:40] System command processed: Exit confirmed
[2024-01-15 14:30:41] ✅ Terminal reset correctly

# Automatic log rotation and size management with improved categorization
```

### **🔄 Error Recovery**

- **Graceful Panic Handling** with complete terminal cleanup
- **🆕 Central Error Processing** - All system errors flow through unified handler
- **Config Validation** with automatic correction
- **File System Error Handling** with fallbacks
- **Layout Failure Recovery** with emergency layouts
- **🆕 Terminal State Recovery** - Enhanced cursor and color reset on exit
- **🆕 Type-Safe Operations** - Compiler-verified state transitions

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

### **Current Structure (v0.3.1)**

```bash
src/
├── core/           # Core logic & configuration
│   ├── config.rs   # TOML config with enhanced theme system
│   ├── error.rs    # Comprehensive error handling
│   ├── constants.rs # Application constants
│   └── prelude.rs  # Common imports with enhanced traits
├── ui/             # Advanced terminal UI
│   ├── screen.rs   # 🆕 Centralized command processing with simplified flow
│   ├── terminal.rs # Enhanced terminal initialization
│   ├── cursor.rs   # 🆕 Unified cursor system (input/output)
│   ├── viewport.rs # 🆕 Advanced scroll & layout management
│   ├── widget.rs   # Enhanced UI widget traits
│   └── color.rs    # 🆕 Anti-flicker color system with O(1) lookup
├── input/          # Enhanced input handling system
│   ├── keyboard.rs # 🆕 Improved keyboard with better filtering
│   ├── state.rs    # 🆕 Central system command processor with type-safe confirmations
│   └── mod.rs      # Optimized event loop
├── output/         # Enhanced display & logging
│   └── display.rs  # 🆕 Advanced message display with viewport integration
├── commands/       # Streamlined command system
│   ├── clear/      # Clear command
│   ├── exit/       # 🆕 Enhanced exit with central confirmation
│   ├── history/    # 🆕 Enhanced history management with central confirmation
│   ├── lang/       # Language switching
│   ├── log_level/  # Log level control
│   ├── restart/    # 🆕 Enhanced restart with central confirmation
│   ├── theme/      # 🆕 Enhanced live theme system
│   ├── version/    # Version display
│   ├── command.rs  # Command trait
│   ├── handler.rs  # 🆕 Enhanced command processing
│   └── registry.rs # Command registry
├── setup/          # Auto-configuration
│   └── setup_toml.rs # 🆕 Enhanced config with sorted themes
└── i18n/           # Enhanced internationalization
    ├── mod.rs      # 🆕 Centralized translation engine with improved caching
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

### **📐 Code Quality Checks**

```bash
# Zero warnings guarantee
cargo clippy --all-targets --all-features
cargo check --all-targets
cargo test --all-features

# Specific component tests
cargo test central_command_system
cargo test color_system_tests
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
cargo test central_command_processor
```

### **🎨 UI System Tests**

```bash
# Enhanced system tests
cargo test viewport_calculations
cargo test scroll_management
cargo test cursor_positioning
cargo test theme_switching
cargo test terminal_compatibility
cargo test color_system_performance
cargo test confirmation_system
cargo test startup_sequence
```

---

## 📊 Version History

### **v0.3.1 (Current) - Central Command Architecture**

**🏗️ Major Architectural Improvements:**

- **🎯 Central Confirmation System** - Complete overhaul with type-safe confirmation processing
- **⚡ One-Line System Commands** - Reduced from 5-7 code paths to single execution pipeline
- **🌈 Anti-Flicker Color System** - Pre-compiled color mappings with O(1) lookup performance
- **🎬 Professional Startup Experience** - Restored and enhanced startup messages with color coding
- **🛡️ Type-Safe Operations** - Eliminated string-based states with robust enum architecture

**🔧 Core System Refinements:**

- **📁 Complete state.rs Overhaul** - Central system command processor with unified confirmation handling
- **🎨 Enhanced color.rs** - Anti-flicker engine with pre-compiled display-to-color mappings
- **🖥️ Optimized screen.rs** - Simplified command flow with better error handling and immediate execution
- **⚙️ Multiple command.rs Improvements** - Enhanced exit, restart, and history commands with central processing
- **🌍 Improved Startup Sequence** - Professional welcome messages with language adaptation

**📈 Performance & Reliability:**

- **⚡ 60-80% Faster Color Processing** - Zero computational overhead for UI color assignments
- **🚫 Race Condition Elimination** - Central processor prevents timing issues and multiple execution paths
- **🎯 Immediate Command Execution** - No more "**EXIT**" or "**RESTART**" display bugs
- **🛡️ Comprehensive Error Handling** - Type-safe operations with compiler-verified state transitions

### **v0.3.0 - Code Optimization & Performance**

**🔧 Major Code Architecture Improvements:**

- **📦 17.6% Code Reduction** - From 289,700 to 238,817 characters
- **🧹 Complete Code Cleanup** - Removed redundant structures and debug code
- **⚡ Performance Optimizations** - Streamlined rendering and input processing
- **🎯 Focused Module Structure** - Consolidated and simplified APIs
- **🔄 Enhanced Widget System** - Improved trait implementations

### **v0.2.9 - Screen & Cursor System Complete**

**🎉 Major Features:**

- 🖥️ Complete screen management overhaul with robust viewport handling
- 📜 Advanced scroll system with smooth navigation and auto-scroll detection
- 🎨 Terminal cursor integration - Real terminal cursor synchronized with text
- 🔄 Enhanced live theme updates with complete UI state preservation
- 🛡️ Bulletproof input state management with backup/restore functionality

### **v0.2.8 - Foundation Complete**

**🎉 Major Features:**

- 📝 Persistent message logging to `.rss/rush.logs`
- 📚 Persistent command history in `.rss/rush.history`
- 🛡️ Advanced terminal compatibility with escape sequence detection
- 🔧 Enhanced error handling throughout codebase
- 🧹 Code architecture cleanup (removed performance module)

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

**Rush Sync Server v0.3.1** maintains exceptional standards:

- ✅ **Zero Clippy Warnings** (all lints passing)
- ✅ **Zero Cargo Check Errors** (clean compilation)
- ✅ **Memory Safe** (Rust guarantees + manual verification)
- ✅ **Thread Safe** (proper async/sync boundaries)
- ✅ **Comprehensive Error Handling** (Result types throughout)
- ✅ **Clean Architecture** (modular design patterns)
- ✅ **Extensive Testing** (unit + integration tests)
- ✅ **Documentation Coverage** (all public APIs documented)
- ✅ **🆕 Central Command Architecture** (type-safe system operations)
- ✅ **🆕 Anti-Flicker Performance** (O(1) color processing)
- ✅ **🆕 Race-Condition Free** (centralized state management)
- ✅ **Cross-Platform Compatibility** (tested on macOS, Linux, Windows)

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
   - Follow the central command architecture pattern

2. **Internationalization:**

   - Add i18n support for all new user-facing text
   - Update both `en.json` and `de.json` files
   - Test language switching functionality
   - Use centralized color categories for consistent theming

3. **Configuration:**

   - Update config validation for new parameters
   - Provide sensible defaults and auto-correction
   - Test all theme configurations including cursor settings
   - Follow type-safe patterns for system operations

4. **Documentation:**
   - Update README.md for new features
   - Add inline documentation for public APIs
   - Include usage examples
   - Document terminal compatibility notes
   - Explain central command architecture decisions

---

## 📞 Contact & Support

- **Primary Contact:** 📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)
- **GitHub Repository:** [LEVOGNE/rush.sync.server](https://github.com/LEVOGNE/rush.sync.server)
- **Issues & Bug Reports:** [GitHub Issues](https://github.com/LEVOGNE/rush.sync.server/issues)
- **Feature Requests:** [GitHub Discussions](https://github.com/LEVOGNE/rush.sync.server/discussions)

---

\_Rush Sync Server v0.3.1 - Central command architecture with type-safe confirmations. Anti-flicker color system, professional startup
