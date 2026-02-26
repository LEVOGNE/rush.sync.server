use crate::memory::{register_embedded, ResourceKind};

pub const APP_TITLE: &str = "RUSH SYNC SERVER";
pub const DEFAULT_BUFFER_SIZE: usize = 1000;
pub const DEFAULT_POLL_RATE: u64 = 16;
pub const MIN_POLL_RATE: u64 = 16;
pub const MAX_POLL_RATE: u64 = 1000;
pub const DOUBLE_ESC_THRESHOLD: u64 = 250;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Registriert die Konstanten im Memory-Manager
pub fn register_constants_to_memory() {
    register_embedded(
        "core:constant:app_title@v1",
        ResourceKind::EmbeddedAsset,
        APP_TITLE.len() as u64,
    );

    register_embedded(
        "core:constant:default_buffer_size@v1",
        ResourceKind::EmbeddedAsset,
        std::mem::size_of_val(&DEFAULT_BUFFER_SIZE) as u64,
    );

    register_embedded(
        "core:constant:default_poll_rate@v1",
        ResourceKind::EmbeddedAsset,
        std::mem::size_of_val(&DEFAULT_POLL_RATE) as u64,
    );

    register_embedded(
        "core:constant:min_poll_rate@v1",
        ResourceKind::EmbeddedAsset,
        std::mem::size_of_val(&MIN_POLL_RATE) as u64,
    );

    register_embedded(
        "core:constant:max_poll_rate@v1",
        ResourceKind::EmbeddedAsset,
        std::mem::size_of_val(&MAX_POLL_RATE) as u64,
    );

    register_embedded(
        "core:constant:double_esc_threshold@v1",
        ResourceKind::EmbeddedAsset,
        std::mem::size_of_val(&DOUBLE_ESC_THRESHOLD) as u64,
    );

    register_embedded(
        "core:constant:version@v1",
        ResourceKind::EmbeddedAsset,
        VERSION.len() as u64,
    );
}
