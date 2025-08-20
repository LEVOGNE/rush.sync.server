#[derive(Debug, Clone)]
pub struct Viewport {
    terminal_width: u16,
    terminal_height: u16,
    output_area: LayoutArea,
    input_area: LayoutArea,
    content_height: usize,
    window_height: usize,
    scroll_offset: usize,
    auto_scroll_enabled: bool,
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
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let mut viewport = Self {
            terminal_width: terminal_width.max(40),
            terminal_height: terminal_height.max(10),
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

    pub fn update_terminal_size(&mut self, width: u16, height: u16) -> bool {
        let new_width = width.max(self.min_terminal_width);
        let new_height = height.max(self.min_terminal_height);
        let changed = self.terminal_width != new_width || self.terminal_height != new_height;

        if changed {
            self.terminal_width = new_width;
            self.terminal_height = new_height;
            self.calculate_layout();
            self.adjust_scroll_after_resize();
        }
        changed
    }

    fn calculate_layout(&mut self) {
        // Validate and fix dimensions
        if self.terminal_width < 10 || self.terminal_height < 5 {
            self.terminal_width = self.terminal_width.max(10);
            self.terminal_height = self.terminal_height.max(5);
        }

        let margin = 1u16;
        let available_height = self.terminal_height.saturating_sub(margin * 2);

        // Calculate heights with safety checks
        let input_height = match available_height {
            h if h >= 5 => 3,
            h if h >= 3 => 2,
            _ => 2,
        }
        .min(available_height.saturating_sub(1));

        let output_height = available_height.saturating_sub(input_height).max(1);

        // Create layout areas with emergency fallback
        if input_height < 2 || output_height < 1 {
            self.create_emergency_layout(margin);
        } else {
            self.create_normal_layout(margin, output_height, input_height);
        }

        self.window_height = output_height.max(1) as usize;
        self.validate_layout();
    }

    fn create_emergency_layout(&mut self, margin: u16) {
        let width = self.terminal_width.saturating_sub(margin * 2).max(1);
        self.output_area = LayoutArea::new(
            margin,
            margin,
            width,
            self.terminal_height.saturating_sub(3).max(1),
        );
        self.input_area = LayoutArea::new(margin, self.output_area.height + margin, width, 2);
    }

    fn create_normal_layout(&mut self, margin: u16, output_height: u16, input_height: u16) {
        let width = self.terminal_width.saturating_sub(margin * 2).max(1);
        self.output_area = LayoutArea::new(margin, margin, width, output_height);
        self.input_area = LayoutArea::new(margin, margin + output_height, width, input_height);
    }

    fn validate_layout(&mut self) {
        if !self.output_area.is_valid() || !self.input_area.is_valid() {
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
    }

    // Scroll operations - simplified and consolidated
    pub fn scroll_up(&mut self, lines: usize) {
        if lines > 0 {
            self.disable_auto_scroll();
        }
        self.scroll_offset = self.scroll_offset.saturating_sub(lines.max(1));
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines.max(1));
        self.clamp_scroll_offset();
        if self.is_at_bottom() {
            self.enable_auto_scroll();
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.disable_auto_scroll();
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
        self.auto_scroll_enabled = true;
    }

    pub fn page_up(&mut self) {
        self.scroll_up(self.window_height.saturating_sub(1).max(1));
    }

    pub fn page_down(&mut self) {
        self.scroll_down(self.window_height.saturating_sub(1).max(1));
    }

    // Content and auto-scroll management
    pub fn update_content_height(&mut self, new_content_height: usize) {
        self.content_height = new_content_height;
        self.clamp_scroll_offset();
    }

    pub fn update_content_height_silent(&mut self, new_content_height: usize) {
        self.content_height = new_content_height;
        self.clamp_scroll_offset();
    }

    pub fn set_scroll_offset_direct_silent(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.max_scroll_offset());
    }

    pub fn enable_auto_scroll_silent(&mut self) {
        self.auto_scroll_enabled = true;
    }

    pub fn force_auto_scroll(&mut self) {
        self.enable_auto_scroll_silent();
        self.scroll_to_bottom();
    }

    pub fn set_scroll_offset_direct(&mut self, offset: usize) {
        self.scroll_offset = offset;
        self.clamp_scroll_offset();
    }

    pub fn enable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = true;
    }

    pub fn disable_auto_scroll(&mut self) {
        self.auto_scroll_enabled = false;
    }

    // View calculations
    pub fn get_visible_range(&self) -> (usize, usize) {
        if self.content_height == 0 || self.window_height == 0 {
            return (0, 0);
        }
        let start = self.scroll_offset;
        let end = (start + self.window_height).min(self.content_height);
        (start, end)
    }

    // Getters - streamlined
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

    pub fn is_usable(&self) -> bool {
        self.terminal_width >= self.min_terminal_width
            && self.terminal_height >= self.min_terminal_height
            && self.output_area.is_valid()
            && self.input_area.is_valid()
    }

    pub fn debug_info(&self) -> String {
        format!("Viewport: {}x{}, output: {}x{}+{}+{}, input: {}x{}+{}+{}, content: {}, window: {}, offset: {}, auto: {}, at_bottom: {}, max_offset: {}",
            self.terminal_width, self.terminal_height,
            self.output_area.width, self.output_area.height, self.output_area.x, self.output_area.y,
            self.input_area.width, self.input_area.height, self.input_area.x, self.input_area.y,
            self.content_height, self.window_height, self.scroll_offset, self.auto_scroll_enabled,
            self.is_at_bottom(), self.max_scroll_offset())
    }

    pub fn short_debug(&self) -> String {
        format!(
            "{}x{}, content: {}, offset: {}",
            self.terminal_width, self.terminal_height, self.content_height, self.scroll_offset
        )
    }

    // Event handling - consolidated
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

    // Private helpers - streamlined
    fn max_scroll_offset(&self) -> usize {
        self.content_height.saturating_sub(self.window_height)
    }

    fn is_at_bottom(&self) -> bool {
        let max_offset = self.max_scroll_offset();
        self.scroll_offset >= max_offset || max_offset == 0
    }

    fn clamp_scroll_offset(&mut self) {
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }

    fn adjust_scroll_after_resize(&mut self) {
        if self.auto_scroll_enabled {
            self.scroll_to_bottom();
        } else {
            self.clamp_scroll_offset();
        }
    }
}
