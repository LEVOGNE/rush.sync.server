// use rush_sync_server::embedded::{register_dir, TEMPLATES_DIR};
// use rush_sync_server::memory::{register_embedded, ResourceKind};

// const BOOT_SENTINEL: &[u8] = b".";

// pub fn early_boot() {
//     // 1 Byte – bleibt wie gehabt
//     register_embedded(
//         "boot:sentinel@v1",
//         ResourceKind::EmbeddedAsset,
//         BOOT_SENTINEL.len() as u64,
//     );

//     // NEU: kompletten Templates-Ordner automatisch registrieren
//     // => legt IDs wie "asset:tpl:rss/favicon.svg@v1", "asset:tpl:rss/_reset.css@v1", ...
//     register_dir(&TEMPLATES_DIR, "asset:tpl");

//     #[cfg(debug_assertions)]
//     {
//         rush_sync_server::memory::debug_dump_to_log();
//     }
// }
