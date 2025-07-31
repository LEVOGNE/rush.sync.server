// =====================================================
// FILE: src/ui/viewport.rs - VERBESSERTES SANFTES SCROLLING
// =====================================================

/// Zentrale Viewport-Verwaltung für alle Dimensionen
/// Löst alle Layout-Math-Probleme durch einheitliche Berechnung
use crate::i18n::get_translation;

#[derive(Debug, Clone)]
pub struct Viewport {
    // Terminal-Dimensionen
    terminal_width: u16,
    terminal_height: u16,

    // Layout-Bereiche (absolut)
    output_area: LayoutArea,
    input_area: LayoutArea,

    // Content-Dimensionen
    content_height: usize,
    window_height: usize,

    // Scroll-Position
    scroll_offset: usize,
    auto_scroll_enabled: bool,

    // Safety margins
    min_terminal_height: u16,
    min_terminal_width: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl LayoutArea {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    pub fn as_rect(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }
}

impl Viewport {
    /// Erstellt neuen Viewport mit sicheren Defaults
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let mut viewport = Self {
            terminal_width: terminal_width.max(40),   // Minimum 40 Zeichen
            terminal_height: terminal_height.max(10), // Minimum 10 Zeilen
            output_area: LayoutArea::new(0, 0, 0, 0),
            input_area: LayoutArea::new(0, 0, 0, 0),
            content_height: 0,
            window_height: 0,
            scroll_offset: 0,
            auto_scroll_enabled: true,
            min_terminal_height: 10,
            min_terminal_width: 40,
        };

        viewport.calculate_layout();
        viewport
    }

    /// Aktualisiert Terminal-Größe und berechnet Layout neu
    pub fn update_terminal_size(&mut self, width: u16, height: u16) -> bool {
        let new_width = width.max(self.min_terminal_width);
        let new_height = height.max(self.min_terminal_height);

        let changed = self.terminal_width != new_width || self.terminal_height != new_height;

        if changed {
            log::debug!(
                "📐 Viewport resize: {}x{} → {}x{}",
                self.terminal_width,
                self.terminal_height,
                new_width,
                new_height
            );

            self.terminal_width = new_width;
            self.terminal_height = new_height;
            self.calculate_layout();

            // Bei Resize: Scroll-Position anpassen
            self.adjust_scroll_after_resize();
        }

        changed
    }

    /// Berechnet alle Layout-Bereiche (ZENTRAL & ROBUST + PANIC-SAFE)
    fn calculate_layout(&mut self) {
        // ✅ PANIC-SAFE: Validiere Input-Dimensionen
        if self.terminal_width < 10 || self.terminal_height < 5 {
            log::error!(
                "{}",
                get_translation(
                    "viewport.layout.too_small",
                    &[
                        &self.terminal_width.to_string(),
                        &self.terminal_height.to_string()
                    ]
                )
            );
            self.terminal_width = self.terminal_width.max(10);
            self.terminal_height = self.terminal_height.max(5);
        }

        // Sichere Margin-Berechnung
        let margin = 1_u16;
        let available_height = self.terminal_height.saturating_sub(margin * 2);

        // ✅ PANIC-SAFE: Mindest-Größen garantieren
        let min_input_height = 2_u16;
        let min_output_height = 1_u16;

        // Input braucht mindestens 2, optimal 3 Zeilen
        let input_height = if available_height >= 5 {
            3
        } else if available_height >= 3 {
            2
        } else {
            min_input_height
        }
        .min(available_height.saturating_sub(min_output_height));

        let output_height = available_height
            .saturating_sub(input_height)
            .max(min_output_height);

        // ✅ PANIC-SAFE: Final validation
        if input_height < min_input_height || output_height < min_output_height {
            log::error!(
                "{}",
                get_translation(
                    "viewport.layout.failed",
                    &[
                        &input_height.to_string(),
                        &output_height.to_string(),
                        &available_height.to_string()
                    ]
                )
            );

            // Emergency fallback
            let emergency_input = min_input_height;
            let emergency_output = available_height.saturating_sub(emergency_input);

            self.output_area = LayoutArea::new(
                margin,
                margin,
                self.terminal_width.saturating_sub(margin * 2).max(1),
                emergency_output.max(1),
            );
            self.input_area = LayoutArea::new(
                margin,
                margin + emergency_output,
                self.terminal_width.saturating_sub(margin * 2).max(1),
                emergency_input,
            );
        } else {
            // Normal layout
            self.output_area = LayoutArea::new(
                margin,
                margin,
                self.terminal_width.saturating_sub(margin * 2).max(1),
                output_height,
            );

            self.input_area = LayoutArea::new(
                margin,
                margin + output_height,
                self.terminal_width.saturating_sub(margin * 2).max(1),
                input_height,
            );
        }

        // Window-Höhe für Scroll-Berechnungen (panic-safe)
        self.window_height = output_height.max(1) as usize;

        // ✅ PANIC-SAFE: Validierung mit besserer Fehlerbehandlung
        let total_used = self.output_area.height + self.input_area.height + margin * 2;
        if total_used != self.terminal_height {
            log::warn!(
                "{}",
                get_translation(
                    "viewport.layout.mismatch",
                    &[
                        &self.terminal_height.to_string(),
                        &total_used.to_string(),
                        &self.output_area.height.to_string(),
                        &self.input_area.height.to_string(),
                        &(margin * 2).to_string()
                    ]
                )
            );

            // ✅ NICHT PANIKEN - nur loggen und weiter
            if total_used > self.terminal_height + 2 {
                // Toleranz von 2 Zeilen
                log::error!("{}", get_translation("viewport.layout.broken", &[]));

                // Emergency layout
                self.output_area = LayoutArea::new(
                    0,
                    0,
                    self.terminal_width,
                    self.terminal_height.saturating_sub(3),
                );
                self.input_area = LayoutArea::new(
                    0,
                    self.terminal_height.saturating_sub(3),
                    self.terminal_width,
                    3,
                );
                self.window_height = self.output_area.height.max(1) as usize;
            }
        }

        // ✅ FINAL SAFETY: Bereiche müssen gültig sein
        if !self.output_area.is_valid() || !self.input_area.is_valid() {
            log::error!("{}", get_translation("viewport.layout.invalid", &[]));

            self.output_area = LayoutArea::new(
                0,
                0,
                self.terminal_width.max(1),
                self.terminal_height.saturating_sub(2).max(1),
            );
            self.input_area =
                LayoutArea::new(0, self.output_area.height, self.terminal_width.max(1), 2);
            self.window_height = self.output_area.height.max(1) as usize;
        }

        log::trace!(
            "{}",
            get_translation(
                "viewport.layout.calculated",
                &[
                    &self.terminal_width.to_string(),
                    &self.terminal_height.to_string(),
                    &self.output_area.width.to_string(),
                    &self.output_area.height.to_string(),
                    &self.output_area.x.to_string(),
                    &self.output_area.y.to_string(),
                    &self.input_area.width.to_string(),
                    &self.input_area.height.to_string(),
                    &self.input_area.x.to_string(),
                    &self.input_area.y.to_string(),
                    &self.window_height.to_string()
                ]
            )
        );
    }

    /// ✅ DEBUGGING: Content-Höhe Update mit detailliertem Logging
    pub fn update_content_height(&mut self, new_content_height: usize) {
        let old_height = self.content_height;
        let old_max_offset = self.max_scroll_offset();

        self.content_height = new_content_height;

        let new_max_offset = self.max_scroll_offset();

        // ✅ WICHTIG: Scroll-Bounds sicherstellen
        self.clamp_scroll_offset();

        let final_offset = self.scroll_offset;

        log::debug!(
            "📊 Viewport content height updated: {} → {} (window: {}, max_offset: {} → {}, scroll_offset: {})",
            old_height,
            new_content_height,
            self.window_height,
            old_max_offset,
            new_max_offset,
            final_offset
        );

        // ✅ VERIFICATION: Prüfe Konsistenz
        if new_content_height > self.window_height && new_max_offset == 0 {
            log::error!(
                "🚨 Content height inconsistency! Content: {}, Window: {}, but max_offset is 0",
                new_content_height,
                self.window_height
            );
        }

        if final_offset > new_max_offset {
            log::error!(
                "🚨 Scroll offset too high! Offset: {}, Max: {}",
                final_offset,
                new_max_offset
            );
        }
    }

    /// ✅ DIREKTES SCROLL-UP mit besserer Kontrolle
    pub fn scroll_up(&mut self, lines: usize) {
        // ✅ AUTO-SCROLL DEAKTIVIEREN beim manuellen Scrollen
        if lines > 0 {
            self.disable_auto_scroll();
        }

        let old_offset = self.scroll_offset;
        let actual_lines = if lines == 0 { 1 } else { lines }; // Default: 1 Zeile
        self.scroll_offset = self.scroll_offset.saturating_sub(actual_lines);

        log::trace!(
            "🔼 Scroll up: {} → {} (-{} lines)",
            old_offset,
            self.scroll_offset,
            actual_lines
        );
    }

    /// ✅ DIREKTES SCROLL-DOWN mit Auto-Scroll-Reaktivierung
    pub fn scroll_down(&mut self, lines: usize) {
        let old_offset = self.scroll_offset;
        let actual_lines = if lines == 0 { 1 } else { lines }; // Default: 1 Zeile
        self.scroll_offset = self.scroll_offset.saturating_add(actual_lines);

        // ✅ WICHTIG: Clamp vor Auto-Scroll-Check
        self.clamp_scroll_offset();

        // ✅ AUTO-SCROLL reaktivieren wenn am Ende angelangt
        if self.is_at_bottom() {
            self.enable_auto_scroll();
            log::trace!("✅ Auto-scroll re-enabled (reached bottom)");
        }

        log::trace!(
            "🔽 Scroll down: {} → {} (+{} lines, auto_scroll: {})",
            old_offset,
            self.scroll_offset,
            actual_lines,
            self.auto_scroll_enabled
        );
    }

    pub fn scroll_to_top(&mut self) {
        self.disable_auto_scroll();
        self.scroll_offset = 0;
        log::trace!("🔝 Scroll to top");
    }

    /// ✅ DIREKTES Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = self.max_scroll_offset();
        self.enable_auto_scroll();

        log::trace!(
            "🔚 Scroll to bottom: {} → {} (max_offset: {}, content: {}, window: {})",
            old_offset,
            self.scroll_offset,
            self.max_scroll_offset(),
            self.content_height,
            self.window_height
        );
    }

    /// ✅ SILENT VERSION: Content-Höhe Update ohne Logging (Anti-Loop)
    pub fn update_content_height_silent(&mut self, new_content_height: usize) {
        self.content_height = new_content_height;
        self.clamp_scroll_offset();
    }

    /// ✅ SILENT VERSION: Direkte Scroll-Offset-Kontrolle ohne Logging (Anti-Loop)
    pub fn set_scroll_offset_direct_silent(&mut self, offset: usize) {
        self.scroll_offset = offset;
        self.clamp_scroll_offset();
    }

    /// ✅ SILENT VERSION: Auto-Scroll aktivieren ohne Logging (Anti-Loop)
    pub fn enable_auto_scroll_silent(&mut self) {
        self.auto_scroll_enabled = true;
    }

    /// ✅ LEGACY-KOMPATIBILITÄT: Erzwingt Auto-scroll (nutzt jetzt Silent-Methoden)
    pub fn force_auto_scroll(&mut self) {
        self.enable_auto_scroll_silent();
        self.scroll_to_bottom();
    }

    /// ✅ PAGE-SCROLLING Logik
    pub fn page_up(&mut self) {
        let page_size = self.window_height.saturating_sub(1).max(1);
        log::trace!("📄 Page up: {} lines", page_size);
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self) {
        let page_size = self.window_height.saturating_sub(1).max(1);
        log::trace!("📄 Page down: {} lines", page_size);
        self.scroll_down(page_size);
    }

    /// ✅ NEUE METHODE: Direkte Scroll-Offset-Kontrolle (bypass Event-System)
    pub fn set_scroll_offset_direct(&mut self, offset: usize) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = offset;
        self.clamp_scroll_offset();

        log::trace!(
            "📍 Direct scroll offset set: {} → {} (clamped to {})",
            old_offset,
            offset,
            self.scroll_offset
        );
    }

    /// ✅ NEUE METHODE: Auto-Scroll explizit aktivieren
    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = true;
        log::trace!("✅ Auto-scroll enabled");
    }

    /// ✅ NEUE METHODE: Auto-Scroll explizit deaktivieren
    pub fn disable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = false;
        log::trace!("❌ Auto-scroll disabled");
    }

    /// Berechnet sichtbaren Bereich für Messages
    pub fn get_visible_range(&self) -> (usize, usize) {
        if self.content_height == 0 || self.window_height == 0 {
            return (0, 0);
        }

        let start = self.scroll_offset;
        let end = (start + self.window_height).min(self.content_height);

        log::trace!(
            "👁️ Visible range: [{}, {}) of {} (window: {}, offset: {})",
            start,
            end,
            self.content_height,
            self.window_height,
            self.scroll_offset
        );

        (start, end)
    }

    /// Getter für Layout-Bereiche
    pub fn output_area(&self) -> LayoutArea {
        self.output_area
    }

    pub fn input_area(&self) -> LayoutArea {
        self.input_area
    }

    pub fn window_height(&self) -> usize {
        self.window_height
    }

    pub fn content_height(&self) -> usize {
        self.content_height
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn is_auto_scroll_enabled(&self) -> bool {
        self.auto_scroll_enabled
    }

    pub fn terminal_size(&self) -> (u16, u16) {
        (self.terminal_width, self.terminal_height)
    }

    /// Prüft ob Viewport groß genug ist
    pub fn is_usable(&self) -> bool {
        self.terminal_width >= self.min_terminal_width
            && self.terminal_height >= self.min_terminal_height
            && self.output_area.is_valid()
            && self.input_area.is_valid()
    }

    /// ✅ ERWEITERTE Debug-Informationen
    pub fn debug_info(&self) -> String {
        format!(
            "Viewport: {}x{}, output: {}x{}+{}+{}, input: {}x{}+{}+{}, content: {}, window: {}, offset: {}, auto: {}, at_bottom: {}, max_offset: {}",
            self.terminal_width, self.terminal_height,
            self.output_area.width, self.output_area.height, self.output_area.x, self.output_area.y,
            self.input_area.width, self.input_area.height, self.input_area.x, self.input_area.y,
            self.content_height, self.window_height, self.scroll_offset, self.auto_scroll_enabled,
            self.is_at_bottom(), self.max_scroll_offset()
        )
    }

    // ==================== PRIVATE HELPERS ====================

    fn max_scroll_offset(&self) -> usize {
        if self.content_height > self.window_height {
            self.content_height - self.window_height
        } else {
            0
        }
    }

    /// ✅ VERBESSERT: Prüft ob am Ende mit Toleranz
    fn is_at_bottom(&self) -> bool {
        let max_offset = self.max_scroll_offset();
        // ✅ KLEINE TOLERANZ für Floating-Point-Fehler
        self.scroll_offset >= max_offset || max_offset == 0
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    fn adjust_scroll_after_resize(&mut self) {
        // Bei Resize: Versuche relative Position zu behalten
        if self.auto_scroll_enabled {
            self.scroll_to_bottom();
        } else {
            self.clamp_scroll_offset();
        }
    }
}

/// Viewport-Events für Koordination
#[derive(Debug, Clone)]
pub enum ViewportEvent {
    TerminalResized {
        width: u16,
        height: u16,
    },
    ContentChanged {
        new_height: usize,
    },
    ScrollRequest {
        direction: ScrollDirection,
        amount: usize,
    },
    ForceAutoScroll,
}

#[derive(Debug, Clone)]
pub enum ScrollDirection {
    Up,
    Down,
    ToTop,
    ToBottom,
    PageUp,
    PageDown,
}

impl Viewport {
    /// ✅ VERBESSERT: Event-Processing mit detailliertem Logging
    pub fn handle_event(&mut self, event: ViewportEvent) -> bool {
        match event {
            ViewportEvent::TerminalResized { width, height } => {
                self.update_terminal_size(width, height)
            }
            ViewportEvent::ContentChanged { new_height } => {
                self.update_content_height(new_height);
                true
            }
            ViewportEvent::ScrollRequest { direction, amount } => {
                log::trace!(
                    "📜 Processing scroll request: {:?} by {}",
                    direction,
                    amount
                );

                match direction {
                    ScrollDirection::Up => self.scroll_up(amount),
                    ScrollDirection::Down => self.scroll_down(amount),
                    ScrollDirection::ToTop => self.scroll_to_top(),
                    ScrollDirection::ToBottom => self.scroll_to_bottom(),
                    ScrollDirection::PageUp => self.page_up(),
                    ScrollDirection::PageDown => self.page_down(),
                }
                true
            }
            ViewportEvent::ForceAutoScroll => {
                self.force_auto_scroll();
                true
            }
        }
    }
}
