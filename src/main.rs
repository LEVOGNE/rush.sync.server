// ## BEGIN ##
use log::{error, info, warn};
use rush_sync_server::output::logging::AppLogger;
use rush_sync_server::{i18n, run, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Sprache initialisieren (vor dem Logging)
    match i18n::init().await {
        Ok(_) => {
            let success_msg = i18n::get_command_translation("system.startup.language_success", &[]);
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
    if let Err(e) = rush_sync_server::output::logging::init() {
        let logger_error =
            i18n::get_command_translation("system.startup.logger_init_failed", &[&e.to_string()]);
        println!("{}", logger_error);
    }

    // ✅ AUTO-SCROLL-TEST: 30 Logs mit verschiedenen Levels
    for i in 1..=30 {
        match i % 4 {
            0 => info!("AutoScroll-Test {} – INFO", i),
            1 => warn!("AutoScroll-Test {} – WARNUNG", i),
            2 => error!("AutoScroll-Test {} – FEHLER", i),
            _ => AppLogger::log_plain(format!("AutoScroll-Test {} – Plain Text", i)),
        }
    }

    // Starte die Anwendung (deine normale App)
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            let run_error =
                i18n::get_command_translation("system.startup.run_failed", &[&e.to_string()]);
            println!("{}", run_error);
            Err(e)
        }
    }
}
// ## END ##
