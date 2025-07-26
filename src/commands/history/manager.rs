// =====================================================
// FILE: commands/history/manager.rs - FINAL VERSION (ohne Debug)
// =====================================================

#[derive(Debug)]
pub struct HistoryManager {
    entries: Vec<String>,
    position: Option<usize>,
    max_size: usize,
}

impl HistoryManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_size),
            position: None,
            max_size,
        }
    }

    pub fn add_entry(&mut self, entry: String) {
        if entry.trim().is_empty() || self.entries.contains(&entry) {
            return;
        }

        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }

        self.entries.push(entry);
        self.position = None; // Reset position
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
    }

    pub fn reset_position(&mut self) {
        self.position = None;
    }
}
