use rush_sync::{
    i18n, run,
    ui::color::{AppColor, ColorCategory},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Sprache initialisieren (vor dem Logging)
    match i18n::init().await {
        Ok(_) => {
            let msg = "Sprache erfolgreich initialisiert";
            let colored_msg =
                AppColor::from_category(ColorCategory::Info).format_message("INFO", msg);
            println!("{}", colored_msg); // Direkte Ausgabe ohne Logging zunächst
        }
        Err(e) => {
            eprintln!("Sprachinitialisierung fehlgeschlagen: {}", e);
        }
    }

    // Logger initialisieren
    if let Err(e) = rush_sync::output::logging::init() {
        eprintln!("Logger-Initialisierung fehlgeschlagen: {}", e);
    }

    // Starte die Anwendung
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Fehler beim Ausführen: {}", e);
            Err(e)
        }
    }
}
