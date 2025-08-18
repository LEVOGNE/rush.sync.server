// src/main.rs - MINIMAL DEBUG VERSION
use rush_sync_server::core::config::Config;
use rush_sync_server::ui::screen::ScreenManager;
use rush_sync_server::{i18n, Result};
//use std::fs::OpenOptions;
use std::io::Write;

//const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // ✅ SCHRITT 1: DIREKT IN DATEI SCHREIBEN (ohne Logger)
    test_direct_file_write();

    // ✅ SCHRITT 2: Einfachster Logger
    init_simple_logger();

    // ✅ SCHRITT 3: In Datei schauen
    check_log_file();

    // Panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // ✅ DIRECT FILE WRITE bei Panic
        write_to_file("❌ PANIC OCCURRED!", &format!("{}", panic_info));
    }));

    // Sprache initialisieren (SILENT)
    match i18n::init().await {
        Ok(_) => {
            log::info!("✅ i18n initialized successfully");
        }
        Err(e) => {
            log::error!("❌ i18n initialization failed: {}", e);
        }
    }

    // Config laden
    let config = Config::load_with_messages(false).await?;
    log::info!("✅ Config loaded successfully");

    // Screen Manager starten
    log::info!("🖥️ Starting ScreenManager...");
    let mut screen = ScreenManager::new(&config).await?;

    log::info!("🎯 Entering main loop...");
    screen.run().await
}

fn test_direct_file_write() {
    write_to_file(
        "🧪 DIRECT TEST",
        "Dies sollte definitiv in der Datei stehen!",
    );
}

fn write_to_file(prefix: &str, message: &str) {
    let log_path = get_log_file_path();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {} {}\n", timestamp, prefix, message);

    let result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut file| file.write_all(log_line.as_bytes()));

    if result.is_err() {
        eprintln!("❌ FAILED TO WRITE TO LOG FILE: {:?}", log_path);
    }
}

fn init_simple_logger() {
    use log::{Level, LevelFilter, Metadata, Record};

    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Debug
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let message = format!("🔍 LOG: [{}] {}", record.level(), record.args());
                write_to_file("📋", &message);
            }
        }

        fn flush(&self) {}
    }

    // ✅ Logger registrieren
    if log::set_boxed_logger(Box::new(SimpleLogger)).is_ok() {
        log::set_max_level(LevelFilter::Debug);
        write_to_file("✅", "Logger initialized successfully");
    } else {
        write_to_file("❌", "Failed to initialize logger");
    }
}

fn check_log_file() {
    let log_path = get_log_file_path();

    if log_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            let line_count = content.lines().count();
            write_to_file("📊", &format!("Log file has {} lines", line_count));
        } else {
            write_to_file("❌", "Failed to read log file");
        }
    } else {
        write_to_file("❌", "Log file does not exist!");
    }
}

fn get_log_file_path() -> std::path::PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(base_dir) = exe_path.parent() {
            let log_dir = base_dir.join(".rss");
            let _ = std::fs::create_dir_all(&log_dir);
            return log_dir.join("rush.debug");
        }
    }
    std::path::PathBuf::from("rush.debug")
}
