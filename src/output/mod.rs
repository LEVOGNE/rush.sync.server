// src/output/mod.rs
pub mod display;
pub mod logging;
pub mod scroll;

// Legacy re-exports für Kompatibilität
pub use display::{create_output_widget, MessageDisplay};
