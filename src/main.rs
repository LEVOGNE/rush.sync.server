use rush_sync_server::core::config::Config;
use rush_sync_server::ui::screen::ScreenManager;
use rush_sync_server::{i18n, Result};
use std::io::Write;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file (silently ignore if missing)
    let _ = dotenvy::dotenv();

    // --hash-key CLI command: hash a plaintext API key and exit
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(pos) = args.iter().position(|a| a == "--hash-key") {
            if let Some(plaintext) = args.get(pos + 1) {
                println!("{}", rush_sync_server::core::api_key::hash_api_key(plaintext));
                std::process::exit(0);
            } else {
                eprintln!("Usage: rush-sync --hash-key <your-api-key>");
                std::process::exit(1);
            }
        }
    }

    // 0) Bootstrap
    #[cfg(feature = "memory")]
    {
        let _s = rush_sync_server::memory::begin_scope("phase:bootstrap@v1");
        rush_sync_server::embedded::register_all_src();
        rush_sync_server::core::constants::register_constants_to_memory();
    }

    let headless = std::env::args().any(|a| a == "--headless" || a == "--daemon");

    // 1) Logger
    setup_panic_handler(headless);
    setup_logger();

    // 2) i18n
    i18n::init()
        .await
        .map_err(|e| log::error!("i18n failed: {e}"))
        .ok();

    // 3) Server-System
    log::info!("Initializing server system...");
    rush_sync_server::server::shared::initialize_server_system().await?;

    if headless {
        run_headless().await
    } else {
        run_tui().await
    }
}

async fn run_tui() -> Result<()> {
    let config = Config::load_with_messages(false).await?;
    let mut screen = ScreenManager::new(&config).await?;

    log::info!("Starting application...");
    let result = screen.run().await;

    log::info!("Shutting down...");
    if let Err(e) = rush_sync_server::server::shared::shutdown_all_servers_on_exit().await {
        log::error!("Cleanup error: {e}");
    }

    result
}

async fn run_headless() -> Result<()> {
    log::info!("Rush Sync Server starting in headless mode...");

    // Auto-start servers that were previously running
    match rush_sync_server::server::shared::auto_start_servers().await {
        Ok(started) => {
            if !started.is_empty() {
                log::info!("Auto-started {} servers", started.len());
            }
        }
        Err(e) => log::warn!("Auto-start failed: {}", e),
    }

    log::info!("Headless mode active. Press Ctrl+C to stop.");

    // Wait for SIGTERM/SIGINT
    tokio::signal::ctrl_c().await.ok();

    log::info!("Shutdown signal received...");
    if let Err(e) = rush_sync_server::server::shared::shutdown_all_servers_on_exit().await {
        log::error!("Cleanup error: {e}");
    }

    log::info!("Shutdown complete.");
    Ok(())
}

fn setup_panic_handler(headless: bool) {
    std::panic::set_hook(Box::new(move |panic_info| {
        if !headless {
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show
            );
        }

        write_debug_log("PANIC", &format!("{}", panic_info));
        eprintln!("PANIC: {}", panic_info);
    }));
}

fn setup_logger() {
    struct DebugLogger;

    impl log::Log for DebugLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Debug
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                write_debug_log(&record.level().to_string(), &record.args().to_string());
            }
        }

        fn flush(&self) {}
    }

    if log::set_boxed_logger(Box::new(DebugLogger)).is_ok() {
        log::set_max_level(log::LevelFilter::Debug);
    }
}

fn write_debug_log(level: &str, message: &str) {
    let log_path = get_debug_log_path();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);

    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut file| file.write_all(log_line.as_bytes()));
}

fn get_debug_log_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join(".rss").join("rush.debug")))
        .unwrap_or_else(|| PathBuf::from("rush.debug"))
}
