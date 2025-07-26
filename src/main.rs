// main.rs - VOLLSTÄNDIG INTERNATIONALISIERT
use rush_sync::{i18n, run, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Sprache initialisieren (vor dem Logging)
    match i18n::init().await {
        Ok(_) => {
            // ✅ NACH i18n::init() - können wir Übersetzungen verwenden
            let success_msg = i18n::get_command_translation("system.startup.language_success", &[]);
            println!("{}", success_msg);
        }
        Err(e) => {
            // ✅ PRE-i18n - Generic English mit manueller ANSI-Farbe (da noch keine Übersetzungen verfügbar)
            println!(
                "\x1B[31m[ERROR] Language initialization failed: {}\x1B[0m",
                e
            );
        }
    }

    // Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        // ✅ POST-i18n - können Übersetzungen verwenden
        let logger_error =
            i18n::get_command_translation("system.startup.logger_init_failed", &[&e.to_string()]);
        println!("{}", logger_error);
    }

    // Starte die Anwendung
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            // ✅ POST-i18n - können Übersetzungen verwenden
            let run_error =
                i18n::get_command_translation("system.startup.run_failed", &[&e.to_string()]);
            println!("{}", run_error);
            Err(e)
        }
    }
}
