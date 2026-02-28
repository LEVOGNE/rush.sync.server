use crate::core::prelude::*;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Runtime-safe config loader
pub fn get_config() -> Result<Config> {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(Config::load()))
}

/// Safe read lock acquisition with context for error messages
pub fn read_lock<'a, T>(lock: &'a RwLock<T>, context: &str) -> Result<RwLockReadGuard<'a, T>> {
    lock.read().map_err(|e| {
        log::error!("{} read lock poisoned: {}", context, e);
        AppError::Validation(format!("{} lock poisoned", context))
    })
}

static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Get the base directory (parent of the executable), cached via OnceLock
pub fn get_base_dir() -> Result<PathBuf> {
    Ok(BASE_DIR
        .get_or_init(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        })
        .clone())
}

/// Safe write lock acquisition with context for error messages
pub fn write_lock<'a, T>(lock: &'a RwLock<T>, context: &str) -> Result<RwLockWriteGuard<'a, T>> {
    lock.write().map_err(|e| {
        log::error!("{} write lock poisoned: {}", context, e);
        AppError::Validation(format!("{} lock poisoned", context))
    })
}

/// Escape HTML special characters to prevent XSS
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
