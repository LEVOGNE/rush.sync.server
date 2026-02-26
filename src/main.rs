use rush_sync_server::core::config::Config;
use rush_sync_server::embedded;
use rush_sync_server::ui::screen::ScreenManager;
use rush_sync_server::{i18n, Result};
use std::io::Write;
use std::path::PathBuf;

use rush_sync_server::memory;

#[tokio::main]
async fn main() -> Result<()> {
    // 0) Bootstrap
    {
        let _s = memory::begin_scope("phase:bootstrap@v1");
        embedded::register_all_src(); // oder register_all_src_filtered()
        rush_sync_server::core::constants::register_constants_to_memory();
    }

    // 1) Logger
    {
        let _s = memory::begin_scope("phase:logger_setup@v1");
        setup_panic_handler();
        setup_logger();
    }

    // 2) i18n
    {
        let _s = memory::begin_scope("phase:i18n_init@v1");
        i18n::init()
            .await
            .map_err(|e| log::error!("i18n failed: {e}"))
            .ok();
    }

    // 3) Server-System
    {
        let _s = memory::begin_scope("phase:server_init@v1");
        log::info!("Initializing server system...");
        rush_sync_server::server::shared::initialize_server_system().await?;
    }

    // 4) TUI
    let config = {
        let _s = memory::begin_scope("phase:config_load@v1");
        Config::load_with_messages(false).await?
    };
    let mut screen = {
        let _s = memory::begin_scope("phase:tui_init@v1");
        ScreenManager::new(&config).await?
    };

    // 5) Run
    log::info!("Starting application...");
    let result = screen.run().await;

    // 6) Shutdown (einmal!)
    {
        let _s = memory::begin_scope("phase:server_shutdown@v1");
        log::info!("Shutting down...");
        if let Err(e) = rush_sync_server::server::shared::shutdown_all_servers_on_exit().await {
            log::error!("Cleanup error: {e}");
        }
    }

    result
}

fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        write_debug_log("PANIC", &format!("{}", panic_info));

        tokio::spawn(async {
            let _ = rush_sync_server::server::shared::shutdown_all_servers_on_exit().await;
        });
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
