// src/i18n/cache.rs
use crate::color::ColorCategory;
use std::collections::HashMap;

pub struct TranslationCache {
    entries: HashMap<String, (String, ColorCategory)>,
    hits: usize,
    misses: usize,
    max_size: usize,
}

impl TranslationCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            hits: 0,
            misses: 0,
            max_size,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<(String, ColorCategory)> {
        if let Some(value) = self.entries.get(key) {
            self.hits += 1;
            Some(value.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: String, value: (String, ColorCategory)) {
        if self.entries.len() >= self.max_size {
            self.entries.clear();
            log::debug!("Translation cache cleared due to size limit");
        }
        self.entries.insert(key, value);
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }
}
