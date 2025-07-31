use crate::core::prelude::*;
use crate::ui::color::AppColor;
use crate::ui::cursor::UiCursor; // ✅ NUR UiCursor importieren
use crate::ui::viewport::{ScrollDirection, Viewport, ViewportEvent};
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Message {
    pub content: String,
    pub current_length: usize,
    pub timestamp: Instant,
    pub line_count: usize,
    pub typewriter_cursor: Option<UiCursor>,
}

impl Message {
    pub fn new(content: String, typewriter_delay: Duration) -> Self {
        let line_count = 1;

        let initial_length = if typewriter_delay.as_millis() == 0 {
            content.graphemes(true).count()
        } else {
            0
        };

        let typewriter_cursor = if typewriter_delay.as_millis() > 0 {
            Some(UiCursor::for_typewriter())
        } else {
            None
        };

        Self {
            content,
            current_length: initial_length,
            timestamp: Instant::now(),
            line_count,
            typewriter_cursor,
        }
    }

    pub fn calculate_wrapped_line_count(&mut self, viewport: &Viewport) {
        let clean_content = clean_message_for_display(&self.content);

        if clean_content.is_empty() {
            self.line_count = 1;
            return;
        }

        let output_area = viewport.output_area();
        let available_width = (output_area.width as usize).saturating_sub(2);
        let effective_width = available_width.max(10);

        let mut total_lines = 0;

        for line in clean_content.lines() {
            if line.is_empty() {
                total_lines += 1;
            } else {
                let line_chars = line.graphemes(true).count();
                if line_chars == 0 {
                    total_lines += 1;
                } else {
                    let wrapped_lines = ((line_chars - 1) / effective_width) + 1;
                    total_lines += wrapped_lines;
                }
            }
        }

        self.line_count = total_lines.max(1);
    }

    pub fn is_typing(&self) -> bool {
        if let Some(_) = &self.typewriter_cursor {
            let total_length = self.content.graphemes(true).count();
            self.current_length < total_length
        } else {
            false
        }
    }

    pub fn is_cursor_visible(&self) -> bool {
        if let Some(ref cursor) = self.typewriter_cursor {
            cursor.is_visible()
        } else {
            false
        }
    }
}

static EMPTY_STRING: &str = "";

pub struct MessageDisplay {
    messages: Vec<Message>,
    config: Config,
    viewport: Viewport,
    persistent_cursor: UiCursor,
}

impl MessageDisplay {
    pub fn new(config: &Config, terminal_width: u16, terminal_height: u16) -> Self {
        let viewport = Viewport::new(terminal_width, terminal_height);

        // ✅ FIX: Diese 2 Zeilen hinzufügen
        let mut persistent_cursor = UiCursor::for_typewriter();
        persistent_cursor.update_from_config(config);

        Self {
            messages: Vec::with_capacity(config.max_messages),
            config: config.clone(),
            viewport,
            persistent_cursor,
        }
    }

    pub fn update_config(&mut self, new_config: &Config) {
        let old_cursor_config = self.config.theme.output_cursor.clone();
        let new_cursor_config = new_config.theme.output_cursor.clone();
        let old_theme = self.config.current_theme_name.clone();
        let new_theme = new_config.current_theme_name.clone();

        log::info!(
            "📊 MessageDisplay CONFIG UPDATE START: '{}' → '{}' | cursor: '{}' → '{}'",
            old_theme,
            new_theme,
            old_cursor_config,
            new_cursor_config
        );

        // ✅ STEP 1: Update internal config
        self.config = new_config.clone();

        // ✅ STEP 2: FORCE COMPLETE CURSOR RECREATION
        log::info!("🔄 RECREATING persistent cursor with new config...");
        self.persistent_cursor = UiCursor::for_typewriter();
        self.persistent_cursor.update_from_config(new_config);

        // ✅ STEP 4: Handle message buffer
        if self.messages.len() > self.config.max_messages {
            let excess = self.messages.len() - self.config.max_messages;
            self.messages.drain(0..excess);
            self.recalculate_content_height();
        }

        // ✅ FINAL VERIFICATION
        let final_symbol = self.persistent_cursor.get_symbol();
        log::info!(
            "✅ MessageDisplay CONFIG UPDATE COMPLETE: cursor_symbol='{}' | expected_from_config='{}'",
            final_symbol, new_config.theme.output_cursor
        );
    }

    pub fn handle_viewport_event(&mut self, event: ViewportEvent) -> bool {
        let changed = self.viewport.handle_event(event);
        if changed {
            self.recalculate_all_line_counts();
            log::debug!("📐 Viewport updated: {}", self.viewport.debug_info());
        }
        changed
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) -> bool {
        let changed = self.handle_viewport_event(ViewportEvent::TerminalResized { width, height });

        if changed {
            self.viewport.force_auto_scroll();
        }

        changed
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.recalculate_content_height();
        self.viewport.force_auto_scroll();
        self.persistent_cursor.show_cursor();

        log::debug!("🗑️ All messages cleared, persistent cursor remains active");
    }

    pub fn add_message(&mut self, content: String) {
        if content.starts_with("[DEBUG]") || content.starts_with("[TRACE]") {
            eprintln!("STDERR: {}", content);
            return;
        }

        if self.messages.len() >= self.config.max_messages {
            self.messages.remove(0);
        }

        let mut message = Message::new(content, self.config.typewriter_delay);
        message.calculate_wrapped_line_count(&self.viewport);

        self.messages.push(message);
        self.recalculate_content_height_silent();
        self.scroll_to_bottom_direct_silent();
    }

    pub fn update_typewriter(&mut self) {
        self.persistent_cursor.update_blink();

        if self.config.typewriter_delay.as_millis() == 0 {
            return;
        }

        if let Some(last_message) = self.messages.last_mut() {
            let total_length = last_message.content.graphemes(true).count();

            if let Some(ref mut cursor) = last_message.typewriter_cursor {
                cursor.update_blink();
            }

            if last_message.current_length < total_length {
                let elapsed = last_message.timestamp.elapsed();

                if elapsed >= self.config.typewriter_delay {
                    let chars_to_add = if self.config.typewriter_delay.as_millis() <= 5 {
                        let ratio = elapsed.as_millis() as f64
                            / self.config.typewriter_delay.as_millis() as f64;
                        ratio.floor().max(1.0) as usize
                    } else {
                        1
                    };

                    let new_length = (last_message.current_length + chars_to_add).min(total_length);
                    last_message.current_length = new_length;
                    last_message.timestamp = Instant::now();

                    if new_length == total_length {
                        last_message.typewriter_cursor = None;
                        self.viewport.force_auto_scroll();
                        log::trace!("⌨️ Typewriter completed → message cursor removed, persistent cursor continues");
                    }
                }
            }
        }
    }

    fn recalculate_content_height_silent(&mut self) {
        let total_lines = self
            .messages
            .iter()
            .map(|msg| msg.line_count)
            .sum::<usize>();

        self.viewport.update_content_height_silent(total_lines);
    }

    fn scroll_to_bottom_direct_silent(&mut self) {
        self.viewport.enable_auto_scroll_silent();

        let content_height = self.viewport.content_height();
        let window_height = self.viewport.window_height();

        if content_height > window_height {
            let max_offset = content_height - window_height;
            self.viewport.set_scroll_offset_direct_silent(max_offset);
        } else {
            self.viewport.set_scroll_offset_direct_silent(0);
        }
    }

    fn recalculate_all_line_counts(&mut self) {
        for message in &mut self.messages {
            message.calculate_wrapped_line_count(&self.viewport);
        }

        self.recalculate_content_height();

        log::debug!(
            "🔄 Recalculated all line counts for output_width: {}, total messages: {}",
            self.viewport.output_area().width,
            self.messages.len()
        );
    }

    pub fn handle_scroll(&mut self, direction: ScrollDirection, amount: usize) {
        let scroll_amount = match direction {
            ScrollDirection::Up | ScrollDirection::Down => {
                if amount == 0 {
                    1
                } else {
                    amount
                }
            }
            ScrollDirection::PageUp | ScrollDirection::PageDown => 0,
            _ => amount,
        };

        log::trace!("📜 Manual scroll: {:?} by {}", direction, scroll_amount);

        self.handle_viewport_event(ViewportEvent::ScrollRequest {
            direction,
            amount: scroll_amount,
        });
    }

    fn recalculate_content_height(&mut self) {
        let individual_line_counts: Vec<usize> =
            self.messages.iter().map(|msg| msg.line_count).collect();

        let total_lines = individual_line_counts.iter().sum::<usize>();

        log::debug!(
            "📊 Recalculating content height: {} messages → {} total lines",
            self.messages.len(),
            total_lines
        );

        let old_content_height = self.viewport.content_height();
        self.viewport.update_content_height(total_lines);

        let new_content_height = self.viewport.content_height();

        log::debug!(
            "📊 Content height updated: {} → {} (window: {})",
            old_content_height,
            new_content_height,
            self.viewport.window_height()
        );
    }

    pub fn get_content_height(&self) -> usize {
        self.viewport.content_height()
    }

    pub fn get_window_height(&self) -> usize {
        self.viewport.window_height()
    }

    pub fn get_visible_messages(&self) -> Vec<(String, usize, bool, bool, bool)> {
        let window_height = self.viewport.window_height();
        let content_height = self.viewport.content_height();

        if self.messages.is_empty() {
            return vec![(
                EMPTY_STRING.to_string(),
                0,
                false,
                false,
                self.persistent_cursor.is_visible(),
            )];
        }

        if content_height <= window_height {
            let mut result: Vec<(String, usize, bool, bool, bool)> = self
                .messages
                .iter()
                .enumerate()
                .map(|(index, msg)| {
                    let is_last = index == self.messages.len() - 1;
                    (
                        msg.content.clone(),
                        msg.current_length,
                        msg.is_typing(),
                        msg.is_cursor_visible(),
                        is_last && self.persistent_cursor.is_visible(),
                    )
                })
                .collect();

            if let Some(last_msg) = self.messages.last() {
                if !last_msg.is_typing() {
                    result.push((
                        EMPTY_STRING.to_string(),
                        0,
                        false,
                        false,
                        self.persistent_cursor.is_visible(),
                    ));
                }
            }

            return result;
        }

        let mut visible = Vec::new();
        let mut lines_used = 0;

        for (index, message) in self.messages.iter().rev().enumerate() {
            if lines_used + message.line_count <= window_height {
                let is_last = index == 0;
                visible.push((
                    message.content.clone(),
                    message.current_length,
                    message.is_typing(),
                    message.is_cursor_visible(),
                    is_last && self.persistent_cursor.is_visible(),
                ));
                lines_used += message.line_count;
            } else {
                break;
            }
        }

        visible.reverse();

        if let Some((_, _, is_typing, _, _)) = visible.last() {
            if !is_typing && lines_used < window_height {
                visible.push((
                    EMPTY_STRING.to_string(),
                    0,
                    false,
                    false,
                    self.persistent_cursor.is_visible(),
                ));
            }
        }

        visible
    }

    pub fn create_output_widget_for_rendering(
        &self,
    ) -> (
        Vec<(String, usize, bool, bool, bool)>,
        Config,
        crate::ui::viewport::LayoutArea,
        &UiCursor,
    ) {
        let messages = self.get_visible_messages();
        (
            messages,
            self.config.clone(),
            self.viewport.output_area(),
            &self.persistent_cursor,
        )
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    pub fn debug_scroll_status(&self) -> String {
        format!(
            "Scroll: offset={}, content_height={}, window_height={}, auto_scroll={}, at_bottom={}",
            self.viewport.scroll_offset(),
            self.viewport.content_height(),
            self.viewport.window_height(),
            self.viewport.is_auto_scroll_enabled(),
            self.viewport.scroll_offset()
                >= self
                    .viewport
                    .content_height()
                    .saturating_sub(self.viewport.window_height())
        )
    }

    pub fn log(&mut self, level: &str, message: &str) {
        let log_message = format!("[{}] {}", level, message);
        self.add_message(log_message);
    }

    pub fn get_messages_count(&self) -> usize {
        self.messages.len()
    }
}

// UTILITY FUNCTIONS
fn clean_ansi_codes(message: &str) -> String {
    String::from_utf8_lossy(&strip(message.as_bytes()).unwrap_or_default()).into_owned()
}

fn clean_message_for_display(message: &str) -> String {
    clean_ansi_codes(message)
        .replace("__CONFIRM_EXIT__", "")
        .replace("__CLEAR__", "")
        .trim()
        .to_string()
}

fn parse_message_parts(message: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut chars = message.char_indices().peekable();
    let mut start = 0;

    while let Some((i, c)) = chars.peek().cloned() {
        if c == '[' {
            if start < i {
                let text = &message[start..i];
                if !text.trim().is_empty() {
                    parts.push((text.to_owned(), false));
                }
            }

            if let Some(end_idx) = message[i..].find(']') {
                let end = i + end_idx + 1;
                parts.push((message[i..end].to_owned(), true));
                start = end;
                while let Some(&(ci, _)) = chars.peek() {
                    if ci < end {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                parts.push((message[i..].to_owned(), false));
                break;
            }
        } else {
            chars.next();
        }
    }

    if start < message.len() {
        let remaining = &message[start..];
        if !remaining.trim().is_empty() {
            parts.push((remaining.to_owned(), false));
        }
    }

    if parts.is_empty() {
        parts.push((message.to_owned(), false));
    }

    parts
}

fn get_marker_color(marker: &str) -> AppColor {
    let display_category = marker
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_start_matches("cat:")
        .to_lowercase();

    if AppColor::from_any(&display_category).to_name() != "gray" {
        return AppColor::from_any(&display_category);
    }

    let mapped_category = crate::i18n::get_color_category_for_display(&display_category);
    AppColor::from_any(mapped_category)
}

pub fn create_output_widget<'a>(
    messages: &'a [(String, usize, bool, bool, bool)],
    layout_area: crate::ui::viewport::LayoutArea,
    config: &'a Config,
    cursor_state: &'a UiCursor,
) -> Paragraph<'a> {
    let max_lines = layout_area.height as usize;
    let mut lines = Vec::new();

    if max_lines == 0 || layout_area.width == 0 {
        log::warn!(
            "🚨 Invalid layout area: {}x{}",
            layout_area.width,
            layout_area.height
        );
        return Paragraph::new(vec![Line::from(vec![Span::raw("⚠️ INVALID LAYOUT")])]).block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        );
    }

    let safe_max_lines = max_lines.min(1000);

    if messages.is_empty() {
        let empty_lines = vec![Line::from(vec![Span::raw("")]); safe_max_lines];
        return Paragraph::new(empty_lines)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(config.theme.output_bg.into())),
            )
            .wrap(Wrap { trim: true });
    }

    for (
        message_idx,
        (message, current_length, is_typing, msg_cursor_visible, persistent_cursor_visible),
    ) in messages.iter().enumerate()
    {
        let is_last_message = message_idx == messages.len() - 1;

        if message.is_empty() {
            if *persistent_cursor_visible {
                // ✅ USE NEW CURSOR STATE
                lines.push(Line::from(vec![cursor_state.create_cursor_span(config)]));
            } else {
                lines.push(Line::from(vec![Span::raw("")]));
            }
            continue;
        }

        let clean_message = clean_message_for_display(message);
        let message_lines: Vec<&str> = clean_message.lines().collect();

        if message_lines.is_empty() {
            lines.push(Line::from(vec![Span::raw("")]));
        } else {
            for (line_idx, line_content) in message_lines.iter().enumerate() {
                if lines.len() >= safe_max_lines {
                    log::trace!("🛑 Reached safe line limit: {}", safe_max_lines);
                    break;
                }

                let is_last_line = line_idx == message_lines.len() - 1;

                let visible_chars = if is_last_message && is_last_line {
                    let chars_before_this_line: usize = message_lines
                        .iter()
                        .take(line_idx)
                        .map(|l| l.graphemes(true).count() + 1)
                        .sum();

                    let available_for_this_line =
                        current_length.saturating_sub(chars_before_this_line);
                    available_for_this_line.min(line_content.graphemes(true).count())
                } else {
                    line_content.graphemes(true).count()
                };

                let message_parts = parse_message_parts(line_content);
                let mut spans = Vec::new();
                let mut chars_used = 0;

                for (part_text, is_marker) in message_parts {
                    let part_chars = part_text.graphemes(true).count();
                    let part_style = if is_marker {
                        Style::default().fg(get_marker_color(&part_text).into())
                    } else {
                        Style::default().fg(config.theme.output_text.into())
                    };

                    if chars_used >= visible_chars {
                        break;
                    }

                    let chars_needed = visible_chars - chars_used;

                    if chars_needed >= part_chars {
                        spans.push(Span::styled(part_text, part_style));
                        chars_used += part_chars;
                    } else {
                        let graphemes: Vec<&str> = part_text.graphemes(true).collect();
                        spans.push(Span::styled(
                            graphemes
                                .iter()
                                .take(chars_needed)
                                .copied()
                                .collect::<String>(),
                            part_style,
                        ));
                        break;
                    }
                }

                // ✅ IMPROVED: Better cursor logic with new CursorState
                if is_last_message && is_last_line {
                    if *is_typing && *msg_cursor_visible {
                        // ✅ FIX: Use consistent cursor state instead of CursorConfig
                        spans.push(cursor_state.create_cursor_span(config));
                    } else if !*is_typing && *persistent_cursor_visible {
                        // ✅ ALREADY CORRECT: Use persistent cursor state
                        spans.push(cursor_state.create_cursor_span(config));
                    }
                }

                if spans.is_empty() {
                    spans.push(Span::raw(""));
                }

                lines.push(Line::from(spans));
            }
        }

        if lines.len() >= safe_max_lines {
            break;
        }
    }

    while lines.len() < safe_max_lines {
        lines.push(Line::from(vec![Span::raw("")]));
    }

    lines.truncate(safe_max_lines);

    if lines.is_empty() {
        log::error!("🚨 Empty lines vector created!");
        lines.push(Line::from(vec![Span::raw("ERROR: Empty buffer")]));
    }

    log::trace!(
        "✅ Widget created: {} lines, area: {}x{} (with live cursor type support)",
        lines.len(),
        layout_area.width,
        layout_area.height
    );

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        )
        .wrap(Wrap { trim: true })
}
