use rush_sync_server::core::config::Config;
use rush_sync_server::ui::screen::ScreenManager;
use rush_sync_server::{i18n, Result};

// Define the VERSION constant here or import it from your crate
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // NEU: Panic handler für Terminal cleanup
    std::panic::set_hook(Box::new(|panic_info| {
        // Terminal zurücksetzen bei Panic
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // Original panic message ausgeben
        eprintln!("Application panicked: {}", panic_info);
    }));
    // Sprache initialisieren (vor dem Logging)
    match i18n::init().await {
        Ok(_) => {
            // ✅ FIRST-RUN Message - kann bleiben
            let success_msg = i18n::get_translation("system.startup.version", &[VERSION]);
            println!("{}", success_msg);
        }
        Err(e) => {
            println!(
                "\x1B[31m[ERROR] Language initialization failed: {}\x1B[0m",
                e
            );
        }
    }

    // Logger initialisieren
    if let Err(e) = rush_sync_server::output::logging::init().await {
        let logger_error =
            i18n::get_command_translation("system.startup.logger_init_failed", &[&e.to_string()]);
        println!("{}", logger_error);
    }

    // ✅ CONFIG LADEN: show_messages = true NUR EINMALIG beim Start
    // Danach überall Config::load() ohne Messages
    let config = Config::load_with_messages(true).await?;

    // ✅ Ab hier normales Laden ohne Messages
    let mut screen = ScreenManager::new(&config).await?;
    screen.run().await
}
