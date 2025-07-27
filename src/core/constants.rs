// =====================================================
// FILE: src/core/constants.rs - SICHERE PERFORMANCE-DEFAULTS
// =====================================================

// Zentrale Terminal-Konfiguration
pub const APP_TITLE: &str = "RUSH SYNC SERVER";
pub const DEFAULT_BUFFER_SIZE: usize = 100;

// ✅ SICHERE POLL-RATE: 16ms = 60 FPS (nicht schneller!)
pub const DEFAULT_POLL_RATE: u64 = 16; // Vorher: 16, jetzt explizit sicher

// ✅ PERFORMANCE-GRENZEN
pub const MIN_POLL_RATE: u64 = 16; // 60 FPS maximum (Performance-Limit)
pub const MAX_POLL_RATE: u64 = 1000; // 1 FPS minimum
pub const MIN_TYPEWRITER_DELAY: u64 = 1; // Minimum 1ms (Zero vermeiden)
pub const MAX_TYPEWRITER_DELAY: u64 = 2000; // Maximum 2 Sekunden

// ESC-Doppelklick
pub const DOUBLE_ESC_THRESHOLD: u64 = 250;

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ✅ PERFORMANCE-EMPFEHLUNGEN als Dokumentation
/// Empfohlene Werte für optimale Performance:
///
/// **Poll Rate (Event Loop):**
/// - 16ms = 60 FPS (empfohlen für flüssiges UI)
/// - 33ms = 30 FPS (OK für langsamere Systeme)
/// - 1-15ms = NICHT empfohlen (hohe CPU-Last)
/// - 0ms = CRASH (Tokio interval panic)
///
/// **Typewriter Delay:**
/// - 50ms = 20 Zeichen/Sekunde (gut lesbar)
/// - 30ms = 33 Zeichen/Sekunde (schnell)
/// - 100ms = 10 Zeichen/Sekunde (entspannt)
/// - 0ms = Typewriter-Effekt deaktiviert
pub mod performance_guide {
    pub const RECOMMENDED_POLL_RATE: u64 = 16; // 60 FPS
    pub const ACCEPTABLE_POLL_RATE: u64 = 33; // 30 FPS
    pub const SLOW_POLL_RATE: u64 = 50; // 20 FPS

    pub const FAST_TYPEWRITER: u64 = 30; // 33 chars/sec
    pub const NORMAL_TYPEWRITER: u64 = 50; // 20 chars/sec
    pub const SLOW_TYPEWRITER: u64 = 100; // 10 chars/sec
}
