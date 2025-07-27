// =====================================================
// FILE: src/commands/performance/mod.rs - CLEAN EXPORTS
// =====================================================

pub mod command;
pub mod manager;

// ✅ CLEAN EXPORTS - nur was wirklich gebraucht wird
pub use command::PerformanceCommand;
pub use manager::PerformanceManager;
