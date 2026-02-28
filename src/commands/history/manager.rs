use std::path::PathBuf;

#[derive(Debug)]
pub struct HistoryManager {
    entries: Vec<String>,
    position: Option<usize>,
    max_size: usize,
    file_path: PathBuf,
}

impl HistoryManager {
    pub fn new(max_size: usize) -> Self {
        let file_path = Self::get_history_path();

        let mut manager = Self {
            entries: Vec::with_capacity(max_size),
            position: None,
            max_size,
            file_path,
        };

        // Load history from disk immediately
        manager.load_from_file();
        manager
    }

    fn get_history_path() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(base_dir) = exe_path.parent() {
                let history_path = base_dir.join(".rss").join("rush.history");

                // Create directory if it doesn't exist
                if let Some(parent) = history_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                return history_path;
            }
        }
        PathBuf::from("rush.history") // Fallback
    }

    fn load_from_file(&mut self) {
        if !self.file_path.exists() {
            return;
        }

        if let Ok(content) = std::fs::read_to_string(&self.file_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !self.entries.contains(&trimmed.to_string()) {
                    self.entries.push(trimmed.to_string());
                }
            }

            // Trim to max_size
            if self.entries.len() > self.max_size {
                self.entries.drain(0..self.entries.len() - self.max_size);
            }

            log::info!("Loaded {} history entries", self.entries.len());
        }
    }

    fn save_to_file(&self) {
        let content = self.entries.join("\n");
        if let Err(e) = std::fs::write(&self.file_path, content) {
            log::error!("Failed to save history: {}", e);
        }
    }

    pub fn add_entry(&mut self, entry: String) {
        if entry.trim().is_empty() {
            return;
        }

        // Remove duplicates
        self.entries.retain(|e| e != &entry);

        // Append new entry
        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }

        self.entries.push(entry);
        self.position = None;

        // Persist immediately
        self.save_to_file();
    }

    pub fn navigate_previous(&mut self) -> Option<String> {
        if let Some(pos) = self.position {
            if pos > 0 {
                self.position = Some(pos - 1);
                return self.entries.get(pos - 1).cloned();
            }
        } else if !self.entries.is_empty() {
            self.position = Some(self.entries.len() - 1);
            return self.entries.last().cloned();
        }
        None
    }

    pub fn navigate_next(&mut self) -> Option<String> {
        if let Some(pos) = self.position {
            if pos < self.entries.len() - 1 {
                self.position = Some(pos + 1);
                return self.entries.get(pos + 1).cloned();
            } else {
                self.position = None;
                return Some(String::new()); // Empty = clear input
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = None;

        let _ = std::fs::remove_file(&self.file_path);
        log::info!("History cleared");
    }

    pub fn reset_position(&mut self) {
        self.position = None;
    }

    pub fn get_all_entries(&self) -> Vec<String> {
        self.entries.clone()
    }

    pub fn import_entries(&mut self, entries: Vec<String>) {
        for entry in entries {
            if !entry.trim().is_empty() && !self.entries.contains(&entry) {
                self.entries.push(entry);
            }
        }

        if self.entries.len() > self.max_size {
            self.entries.drain(0..self.entries.len() - self.max_size);
        }

        self.save_to_file();
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}
