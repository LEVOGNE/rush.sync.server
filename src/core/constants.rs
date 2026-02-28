pub const APP_TITLE: &str = "RUSH SYNC SERVER";
pub const DEFAULT_BUFFER_SIZE: usize = 1000;
pub const DEFAULT_POLL_RATE: u64 = 16;
pub const MIN_POLL_RATE: u64 = 16;
pub const MAX_POLL_RATE: u64 = 1000;
pub const DOUBLE_ESC_THRESHOLD: u64 = 250;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// System command signals
pub const SIG_CLEAR: &str = "__CLEAR__";
pub const SIG_EXIT: &str = "__EXIT__";
pub const SIG_CONFIRM_EXIT: &str = "__CONFIRM_EXIT__";
pub const SIG_RESTART: &str = "__RESTART__";
pub const SIG_RESTART_FORCE: &str = "__RESTART_FORCE__";
pub const SIG_RESTART_WITH_MSG: &str = "__RESTART_WITH_MSG__";
pub const SIG_CONFIRM_RESTART: &str = "__CONFIRM_RESTART__";
pub const SIG_CLEAR_HISTORY: &str = "__CLEAR_HISTORY__";
pub const SIG_CONFIRM_CLEANUP: &str = "__CLEANUP__";
pub const SIG_CONFIRM_PREFIX: &str = "__CONFIRM:";
pub const SIG_LIVE_THEME_UPDATE: &str = "__LIVE_THEME_UPDATE__";
pub const SIG_THEME_MSG_SEP: &str = "__MESSAGE__";

/// Register constants in the memory manager
#[cfg(feature = "memory")]
pub fn register_constants_to_memory() {
    use crate::memory::{register_embedded, ResourceKind};
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
