# Rush Sync Server

![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-Dual--License-blue)
![Crates.io](https://img.shields.io/crates/v/rush-sync-server)

> **NOTE**: Version `0.2.2` on crates.io has a critical bug in language file loading (`*.json` not embedded correctly).
> Please use **version `0.3.2+`** for a stable release!

**Rush Sync Server** is a professional web server orchestration platform written in Rust. The project features a robust terminal UI with internationalization, theming, command system, and **NEW in v0.3.2**: Full web server management with Actix-Web integration.

---

## Project Vision

Rush Sync Server development phases:

- **Phase 0** ✅: Terminal UI foundation with command system
- **Phase 1** ✅ **NEW**: Dynamic Actix-Web server management
- **Phase 2**: Dev/Prod modes with hot-reloading
- **Phase 3**: Redis integration & secure communication
- **Phase 4**: Centralized logging & automation

---

## What's New in v0.3.2

### **🌐 Complete Web Server Management System**

The new version introduces **professional Actix-Web server orchestration**:

- **🚀 Dynamic Server Creation** - Create unlimited web servers with custom names and ports
- **⚡ Instant Start/Stop Control** - Full lifecycle management of running servers
- **🎯 Smart Port Management** - Automatic port allocation from 8080+ range
- **📊 Real-time Server Status** - Track running/stopped/failed server states
- **🧹 Intelligent Cleanup** - Remove stopped or failed servers from registry
- **🔍 Comprehensive Server Listing** - Chronological overview with status indicators

### **🛠 New Server Management Commands**

```bash
create                    # Create server with auto-generated name and port
create myserver           # Create server with custom name
create 8090              # Create server on specific port
create myserver 8090     # Create server with custom name and port

list                     # Show all servers with status
start <server>           # Start server by name, ID, or index number
stop <server>            # Stop running server gracefully
cleanup                  # Remove stopped servers from registry
cleanup failed           # Remove only failed servers
cleanup all              # Remove both stopped and failed servers
```

### **🎯 Advanced Server Features**

- **Professional Web Interface** - Each server provides HTML status page with server info
- **REST API Endpoints** - `/status`, `/api/info`, `/api/metrics`, `/health` for monitoring
- **Graceful Shutdown** - 5-second timeout with force-stop fallback
- **Server Identification** - Unique UUID-based IDs with short display format
- **Chronological Indexing** - Access servers by creation order (1, 2, 3...)
- **Concurrent Operation** - Multiple servers running simultaneously
- **Memory Efficient** - Single worker per server, optimized for embedded use

### **🔧 Enhanced Core System (from v0.3.1)**

- **Central System Command Architecture** - All confirmations use unified, type-safe processing
- **Anti-Flicker Color Engine** - Pre-compiled display text to color mappings for zero-delay rendering
- **Enhanced Startup Experience** - Professional startup message with color-coded categories
- **Streamlined Command Architecture** - Multiple command files optimized for maintainability
- **Type-Safe Confirmations** - No more string-based confirmation states

---

## 🚀 Installation & Usage

### 📦 **As Binary - Version 0.3.2+**

```bash
# Install from crates.io
cargo install rush-sync-server

# Run the application (terminal UI + server management)
rush-sync
```

### 📚 **As Library - Version 0.3.2+**

Add to your `Cargo.toml`:

```toml
[dependencies]
rush-sync-server = "0.3.2"
tokio = { version = "1.36", features = ["full"] }
```

#### **Quick Start Examples:**

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Run with default configuration (includes server management)
    run().await?;
    Ok(())
}
```

```rust
use rush_sync_server::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Custom configuration with optimized performance
    let mut config = load_config().await?;
    config.poll_rate = std::time::Duration::from_millis(8); // 125 FPS
    config.typewriter_delay = std::time::Duration::from_millis(1); // Ultra-fast

    // Run with enhanced server management
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

## ✅ Current Features (Phase 1 Complete)

### **🌐 Web Server Management (NEW in v0.3.2)**

- **Dynamic Actix-Web Server Creation** - Unlimited servers with UUID-based identification
- **Smart Port Allocation** - Automatic port finding from 8080-8180 range
- **Custom Server Configuration** - Name and port customization support
- **Professional Web Interface** - HTML status pages with server information and API endpoints
- **Real-time Server Monitoring** - Status tracking (Running/Stopped/Failed)
- **Graceful Lifecycle Management** - Clean startup and shutdown processes
- **Concurrent Server Operation** - Multiple servers running simultaneously
- **Registry-based Management** - Persistent server information storage

### **🏗️ Core Foundation (Phase 0 Complete)**

- **Interactive Terminal UI** with asynchronous event loop (Tokio)
- **Central System Commands** with type-safe confirmation processing
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
- **Anti-Flicker Color System** - Zero-delay color processing with pre-compiled mappings
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
- **Central Confirmation System** - Unified, type-safe confirmation processing

### **🌍 Internationalization**

- **Runtime Language Switching** (German/English)
- **Centralized Color Categories** with consistent i18n support
- **Localized Error Messages** and help texts
- **Professional Startup Messages** with language adaptation
- **Extensible Translation System** for future languages

### **⚙️ Configuration & Themes**

- **Smart Config Validation** with auto-correction
- **TOML-based Theme System** with live updates
- **Internal Restart** without process termination
- **Persistent Settings** with automatic backup
- **Enhanced Error Recovery** with comprehensive fallback handling

---

## 💻 Available Commands

### **🌐 Server Management Commands (NEW)**

| Command   | Description             | Examples                                    |
| --------- | ----------------------- | ------------------------------------------- |
| `create`  | Create new web server   | `create`, `create myserver`, `create 8090`  |
| `list`    | Show all servers        | `list`                                      |
| `start`   | Start server            | `start 1`, `start myserver`, `start abc123` |
| `stop`    | Stop server             | `stop 1`, `stop myserver`, `stop abc123`    |
| `cleanup` | Remove inactive servers | `cleanup`, `cleanup failed`, `cleanup all`  |

### **📋 Core System Commands**

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

### **🌐 Server Management Examples**

#### **Creating Servers**

```bash
# Auto-generated name and port
create
# Result: Server created: 'rush-sync-server-001' (ID: abc12345) on Port 8080

# Custom name with auto port
create myapi
# Result: Server created: 'myapi' (ID: def67890) on Port 8081

# Custom port with auto name
create 9000
# Result: Server created: 'rush-sync-server-002' (ID: ghi13579) on Port 9000

# Custom name and port
create myapi 9000
# Result: Custom Server created: 'myapi' (ID: jkl24680) on Port 9000
```

#### **Server Control**

```bash
# Start by index (chronological order)
start 1
# Result: Server 'myapi' successfully started on http://127.0.0.1:9000

# Start by name
start myapi
# Result: Server 'myapi' successfully started on http://127.0.0.1:9000

# Start by ID prefix
start abc123
# Result: Server 'rush-sync-server-001' successfully started on http://127.0.0.1:8080

# List all servers
list
# Result:
# Server List:
#   1. myapi - def67890 (Port: 9000) [Running]
#   2. rush-sync-server-001 - abc12345 (Port: 8080) [Stopped]
#   3. testserver - ghi13579 (Port: 8082) [Failed]

# Stop server
stop myapi
# Result: Server 'myapi' stopped

# Cleanup inactive servers
cleanup
# Result: 1 stopped server removed

cleanup failed
# Result: 1 failed server removed

cleanup all
# Result: 2 stopped servers removed
# 1 failed server removed
```

#### **Web Interface Access**

Each created server provides a professional web interface:

```
http://127.0.0.1:8080/          # Welcome page with server info
http://127.0.0.1:8080/status    # JSON status information
http://127.0.0.1:8080/api/info  # Complete API information
http://127.0.0.1:8080/api/metrics # Server metrics
http://127.0.0.1:8080/health    # Health check endpoint
```

### **Enhanced Confirmation System**

All system commands use the centralized confirmation processor:

```bash
exit                    # Shows: [CONFIRM] Do you really want to exit? (y/n)
y                       # ✅ Exits immediately - no display bugs
n                       # ✅ Shows: "Operation cancelled"

restart                 # Shows: [CONFIRM] Really restart? (y/n)
y                       # ✅ Restarts immediately and cleanly

history -c              # Shows: [CONFIRM] Clear command history? (y/n)
y                       # ✅ History cleared with proper confirmation
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

### **✏️ Text Editing**

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

# Server Management Configuration (NEW)
[server]
port_range_start = 8080     # Starting port for auto-allocation
port_range_end = 8180       # Maximum port for auto-allocation
max_concurrent = 10         # Maximum simultaneous servers
shutdown_timeout = 5        # Graceful shutdown timeout (seconds)

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

# Additional themes: green, blue, yellow available
```

---

## 🔧 Advanced Features

### **🌐 Server Architecture Details**

- **Actix-Web Integration** - Professional-grade web server framework
- **Single Worker Design** - Optimized for embedded server management
- **UUID-based Identification** - Unique server IDs with collision avoidance
- **Graceful Shutdown Protocol** - 5-second timeout with force-stop fallback
- **Memory Efficient Operation** - Minimal resource footprint per server
- **Concurrent Server Support** - Run multiple servers simultaneously
- **Professional Web Interface** - HTML status pages with comprehensive server information

### **🛡️ Terminal Compatibility**

- **macOS Terminal.app** - Optimized sequences with RGB color support
- **iTerm2** - Native color support with multiple sequence types
- **VSCode Terminal** - Standard compatibility with fallback handling
- **tmux Sessions** - Proper tmux-wrapped escape sequence handling
- **Generic Terminals** - Universal fallback support with error recovery

### **🖱️ Advanced Cursor System**

```bash
# Real-time cursor changes in terminal
theme blue    # Terminal cursor becomes blue
theme yellow  # Terminal cursor becomes yellow
theme green   # Terminal cursor becomes green

# Cursor debugging
theme debug dark    # Shows detailed cursor configuration
```

### **Central Command Processing**

```rust
// Example: Type-safe, compiler-verified, zero race conditions
enum SystemAction {
    Exit,
    Restart,
    ClearHistory,
}

match confirmed_action {
    SystemAction::Exit => exit_application(),      # ⚡ Immediate
    SystemAction::Restart => restart_system(),    # ⚡ Clean
    SystemAction::ClearHistory => clear_data(),   # ⚡ Instant
}
```

---

## 🗺 Development Roadmap

### **Phase 2: Dev/Prod & Versioning**

- [ ] Dev mode with hot-reloading for server content
- [ ] Production mode with TLS/HTTPS support
- [ ] Automatic versioning (v1, v2, ...) for server instances
- [ ] File watcher integration with `notify` crate
- [ ] SCSS compilation for web assets

### **Phase 3: Communication & Security**

- [ ] Redis Pub/Sub integration for inter-server communication
- [ ] TLS/HTTPS with `rustls` for secure connections
- [ ] Session caching and management
- [ ] Advanced authentication and authorization
- [ ] Server-to-server communication protocols

### **Phase 4: Logging & Automation**

- [ ] Centralized logging dashboard for all servers
- [ ] Automated deployment and setup scripts
- [ ] WebSocket support for real-time communication
- [ ] Comprehensive integration test suite
- [ ] Performance monitoring and metrics collection

### **Future Considerations**

- [ ] Load balancing between multiple server instances
- [ ] Docker containerization support
- [ ] Kubernetes deployment configurations
- [ ] Web-based management dashboard
- [ ] Plugin system for server extensions

---

## 🧪 Testing & Quality Assurance

### **✅ Code Quality Checks**

```bash
# Zero warnings guarantee
cargo clippy --all-targets --all-features
cargo check --all-targets
cargo test --all-features

# Server management tests
cargo test server_lifecycle
cargo test port_allocation
cargo test concurrent_servers
```

### **🛡️ Security Testing**

```bash
# Input sanitization tests
cargo test escape_sequence_filtering
cargo test server_name_validation
cargo test port_validation

# Server security tests
cargo test graceful_shutdown
cargo test resource_cleanup
cargo test concurrent_access
```

---

## 📊 Version History

### **v0.3.2 (Current) - Complete Server Management**

**🌐 Major Server Management Features:**

- **Complete Actix-Web Integration** - Professional web server creation and management
- **Dynamic Server Lifecycle** - Create, start, stop, and cleanup web servers
- **Smart Port Management** - Automatic port allocation with conflict avoidance
- **Professional Web Interface** - HTML status pages with comprehensive server information
- **REST API Endpoints** - `/status`, `/api/info`, `/api/metrics`, `/health` for monitoring
- **Concurrent Server Support** - Run multiple servers simultaneously with efficient resource usage

**🛠 New Server Commands:**

- **create** - Server creation with custom names and ports
- **list** - Comprehensive server listing with status indicators
- **start/stop** - Full server lifecycle control
- **cleanup** - Intelligent removal of inactive servers

**🔧 Enhanced Architecture:**

- **Server Registry System** - Persistent server information storage
- **UUID-based Identification** - Unique server IDs with short display format
- **Chronological Indexing** - Access servers by creation order
- **Graceful Shutdown Protocol** - Clean server termination with timeout handling

### **v0.3.1 - Central Command Architecture**

- **Central Confirmation System** - Type-safe confirmation processing
- **Anti-Flicker Color System** - Pre-compiled color mappings with O(1) lookup performance
- **Professional Startup Experience** - Enhanced startup messages with color coding
- **Type-Safe Operations** - Eliminated string-based states with robust enum architecture

### **v0.3.0 - Code Optimization & Performance**

- **17.6% Code Reduction** - From 289,700 to 238,817 characters
- **Complete Code Cleanup** - Removed redundant structures and debug code
- **Performance Optimizations** - Streamlined rendering and input processing

---

## 🏆 Code Quality Metrics

**Rush Sync Server v0.3.2** maintains exceptional standards:

- ✅ **Zero Clippy Warnings** (all lints passing)
- ✅ **Zero Cargo Check Errors** (clean compilation)
- ✅ **Memory Safe** (Rust guarantees + manual verification)
- ✅ **Thread Safe** (proper async/sync boundaries)
- ✅ **Comprehensive Error Handling** (Result types throughout)
- ✅ **Clean Architecture** (modular design patterns)
- ✅ **Extensive Testing** (unit + integration tests)
- ✅ **Documentation Coverage** (all public APIs documented)
- ✅ **Central Command Architecture** (type-safe system operations)
- ✅ **Anti-Flicker Performance** (O(1) color processing)
- ✅ **Race-Condition Free** (centralized state management)
- ✅ **Professional Server Management** (Actix-Web integration)
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

**Phase 2 Development:**

- Hot-reloading system for development mode
- TLS/HTTPS integration with rustls
- File watcher integration with notify
- SCSS compilation for web assets

**Server Management Enhancements:**

- Load balancing between server instances
- Advanced monitoring and metrics collection
- WebSocket support for real-time communication
- Docker integration and containerization

**Core Improvements:**

- Additional language translations (Spanish, French, Japanese)
- Advanced theme design and UX improvements
- Performance optimizations and benchmarking
- Cross-platform testing and compatibility

### **📋 Development Guidelines:**

1. **Code Quality:**

   - Ensure zero warnings with `cargo clippy`
   - Add comprehensive tests for new features
   - Maintain memory safety and thread safety
   - Include panic-safe error handling
   - Follow the central command architecture pattern

2. **Server Management:**

   - Test all server lifecycle operations
   - Ensure proper resource cleanup
   - Validate port allocation logic
   - Test concurrent server scenarios

3. **Internationalization:**
   - Add i18n support for all new user-facing text
   - Update both `en.json` and `de.json` files
   - Test language switching functionality
   - Use centralized color categories for consistent theming

---

## 📞 Contact & Support

- **Primary Contact:** 📧 [l.ersen@icloud.com](mailto:l.ersen@icloud.com)
- **GitHub Repository:** [LEVOGNE/rush.sync.server](https://github.com/LEVOGNE/rush.sync.server)
- **Issues & Bug Reports:** [GitHub Issues](https://github.com/LEVOGNE/rush.sync.server/issues)
- **Feature Requests:** [GitHub Discussions](https://github.com/LEVOGNE/rush.sync.server/discussions)

---

_Rush Sync Server v0.3.2 - Professional web server orchestration with Actix-Web integration, dynamic server management, and comprehensive terminal UI._
