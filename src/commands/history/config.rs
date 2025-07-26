// =====================================================
// FILE: commands/history/config.rs
// =====================================================

use crate::core::prelude::*;

#[derive(Debug, Clone)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub save_duplicates: bool,
    pub save_empty: bool,
}

impl HistoryConfig {
    pub fn from_main_config(config: &Config) -> Self {
        Self {
            max_entries: config.max_history,
            save_duplicates: false,
            save_empty: false,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 30,
            save_duplicates: false,
            save_empty: false,
        }
    }
}
