pub struct ScrollState {
    pub offset: usize,
    pub window_height: usize,
    content_height: usize,
    auto_scroll: bool,
    force_scroll: bool, // Neues Flag für erzwungenes Scrollen
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            window_height: 0,
            content_height: 0,
            auto_scroll: true,
            force_scroll: false,
        }
    }

    pub fn update_dimensions(&mut self, window_height: usize, content_height: usize) {
        // Speichere die alten Werte für Vergleiche
        let old_window_height = self.window_height;
        let old_content_height = self.content_height;

        // Aktualisiere die Dimensionen
        self.window_height = window_height;
        self.content_height = content_height;

        // Berechne maximalen Offset
        let max_offset = self.content_height.saturating_sub(self.window_height);

        // Wenn sich die Fensterhöhe vergrößert hat, behalte die relative Position bei
        if window_height > old_window_height && !self.auto_scroll {
            let ratio = self.offset as f64 / old_content_height.max(1) as f64;
            self.offset = (ratio * content_height as f64).round() as usize;
        }

        // Stelle sicher, dass der Offset nicht über das Maximum hinausgeht
        self.offset = self.offset.min(max_offset);

        // Wenn auto_scroll aktiv ist oder force_scroll gesetzt wurde,
        // scrolle zum Ende
        if self.auto_scroll || self.force_scroll {
            self.offset = max_offset;
            self.force_scroll = false;
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.force_scroll = false;
        if self.offset > 0 {
            self.offset = self.offset.saturating_sub(amount);
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.content_height.saturating_sub(self.window_height);

        if self.offset >= max_offset {
            self.auto_scroll = true;
        } else {
            self.auto_scroll = false;
            self.offset = (self.offset + amount).min(max_offset);
        }
        self.force_scroll = false;
    }

    // Neue Methode zum Erzwingen des Auto-Scrolls
    pub fn force_auto_scroll(&mut self) {
        self.force_scroll = true;
        self.auto_scroll = true;
    }

    pub fn get_visible_range(&self) -> (usize, usize) {
        if self.content_height <= self.window_height {
            return (0, self.content_height);
        }

        let start = self.offset;
        let end = (self.offset + self.window_height).min(self.content_height);
        (start, end)
    }

    pub fn can_scroll(&self) -> bool {
        true
    }

    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}
