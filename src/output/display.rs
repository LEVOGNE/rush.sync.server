use crate::core::prelude::*;
use crate::ui::color::AppColor;
use crate::ui::cursor::{CursorKind, UiCursor};
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

#[derive(Debug, Clone)]
struct CachedLine {
    content: String,
    message_index: usize,
    is_partial: bool,
    visible_chars: usize,
}

type RenderData<'a> = (
    Vec<(String, usize, bool, bool, bool)>,
    Config,
    crate::ui::viewport::LayoutArea,
    &'a UiCursor,
);

impl Message {
    pub fn new(content: String, typewriter_delay: Duration) -> Self {
        let (initial_length, typewriter_cursor) = if typewriter_delay.as_millis() == 0 {
            (content.graphemes(true).count(), None)
        } else {
            (0, Some(UiCursor::for_typewriter()))
        };

        Self {
            content,
            current_length: initial_length,
            timestamp: Instant::now(),
            line_count: 1,
            typewriter_cursor,
        }
    }

    pub fn calculate_wrapped_line_count(&mut self, viewport: &Viewport) {
        let clean_content = clean_message_for_display(&self.content);
        if clean_content.is_empty() {
            self.line_count = 1;
            return;
        }

        let effective_width = (viewport.output_area().width as usize)
            .saturating_sub(2)
            .max(10);
        let raw_lines: Vec<&str> = clean_content.lines().collect();

        let lines_to_process = if clean_content.ends_with('\n') {
            let mut lines = raw_lines;
            lines.push("");
            lines
        } else if raw_lines.is_empty() {
            vec![""]
        } else {
            raw_lines
        };

        self.line_count = lines_to_process
            .iter()
            .map(|line| {
                if line.is_empty() {
                    1
                } else {
                    ((line.graphemes(true).count().saturating_sub(1)) / effective_width) + 1
                }
            })
            .sum::<usize>()
            .max(1);
    }

    pub fn is_typing(&self) -> bool {
        self.typewriter_cursor.is_some()
            && self.current_length < self.content.graphemes(true).count()
    }
    pub fn is_cursor_visible(&self) -> bool {
        self.typewriter_cursor
            .as_ref()
            .is_some_and(|c| c.is_visible())
    }
}

pub struct MessageDisplay {
    messages: Vec<Message>,
    line_cache: Vec<CachedLine>,
    cache_dirty: bool,
    config: Config,
    viewport: Viewport,
    persistent_cursor: UiCursor,
}

impl MessageDisplay {
    pub fn new(config: &Config, terminal_width: u16, terminal_height: u16) -> Self {
        Self::log_startup();
        Self {
            messages: Vec::with_capacity(config.max_messages),
            line_cache: Vec::new(),
            cache_dirty: true,
            config: config.clone(),
            viewport: Viewport::new(terminal_width, terminal_height),
            persistent_cursor: UiCursor::from_config(config, CursorKind::Output),
        }
    }

    fn rebuild_line_cache(&mut self) {
        self.line_cache.clear();
        let effective_width = (self.viewport.output_area().width as usize)
            .saturating_sub(2)
            .max(10);

        for (msg_idx, message) in self.messages.iter().enumerate() {
            let visible_content = if message.is_typing() {
                let graphemes: Vec<&str> = message.content.graphemes(true).collect();
                graphemes
                    .iter()
                    .take(message.current_length)
                    .copied()
                    .collect::<String>()
            } else {
                message.content.clone()
            };

            let clean_content = clean_message_for_display(&visible_content);
            let lines: Vec<String> = if clean_content.is_empty() {
                vec![String::new()]
            } else {
                let mut lines: Vec<String> = clean_content.lines().map(|s| s.to_string()).collect();
                if clean_content.ends_with('\n') {
                    lines.push(String::new());
                }
                if lines.is_empty() {
                    lines.push(String::new());
                }
                lines
            };

            for (line_idx, raw_line) in lines.iter().enumerate() {
                if raw_line.is_empty() {
                    self.line_cache.push(CachedLine {
                        content: String::new(),
                        message_index: msg_idx,
                        is_partial: false,
                        visible_chars: 0,
                    });
                } else {
                    let graphemes: Vec<&str> = raw_line.graphemes(true).collect();
                    let mut start = 0;
                    while start < graphemes.len() {
                        let end = (start + effective_width).min(graphemes.len());
                        let wrapped_line = graphemes[start..end].join("");
                        let is_last_chunk = end == graphemes.len();
                        let is_last_line = line_idx == lines.len() - 1;

                        self.line_cache.push(CachedLine {
                            content: wrapped_line.clone(),
                            message_index: msg_idx,
                            is_partial: message.is_typing() && is_last_line && is_last_chunk,
                            visible_chars: wrapped_line.graphemes(true).count(),
                        });
                        start = end;
                    }
                }
            }
        }

        if let Some(last_msg) = self.messages.last() {
            if !last_msg.is_typing() {
                self.line_cache.push(CachedLine {
                    content: String::new(),
                    message_index: self.messages.len(),
                    is_partial: false,
                    visible_chars: 0,
                });
            }
        }

        self.cache_dirty = false;
        self.viewport
            .update_content_height_silent(self.line_cache.len());
    }

    pub fn get_visible_messages(&mut self) -> Vec<(String, usize, bool, bool, bool)> {
        if self.cache_dirty {
            self.rebuild_line_cache();
        }

        let window_height = self.viewport.window_height();
        let scroll_offset = self.viewport.scroll_offset();
        let available_lines = self.line_cache.len().saturating_sub(scroll_offset);
        let lines_to_show = available_lines.min(window_height);
        let visible_start = scroll_offset;
        let visible_end = scroll_offset + lines_to_show;

        let mut result = Vec::new();
        if self.line_cache.is_empty() {
            result.push((
                String::new(),
                0,
                false,
                false,
                self.persistent_cursor.is_visible(),
            ));
            return result;
        }

        for line_idx in visible_start..visible_end {
            if let Some(cached_line) = self.line_cache.get(line_idx) {
                let msg_idx = cached_line.message_index;
                let is_last_line = line_idx == self.line_cache.len() - 1;

                let (is_typing, cursor_visible) = if msg_idx < self.messages.len() {
                    self.messages.get(msg_idx).map_or((false, false), |msg| {
                        (
                            cached_line.is_partial && msg.is_typing(),
                            msg.is_cursor_visible() && cached_line.is_partial,
                        )
                    })
                } else {
                    (false, false)
                };

                let persistent_cursor =
                    is_last_line && !is_typing && self.persistent_cursor.is_visible();
                result.push((
                    cached_line.content.clone(),
                    cached_line.visible_chars,
                    is_typing,
                    cursor_visible,
                    persistent_cursor,
                ));
            }
        }

        while result.len() < window_height {
            result.push((String::new(), 0, false, false, false));
        }
        result
    }

    pub fn add_message(&mut self, content: String) {
        self.add_message_with_typewriter(content, true);
    }
    pub fn add_message_instant(&mut self, content: String) {
        self.add_message_with_typewriter(content, false);
    }

    fn add_message_with_typewriter(&mut self, content: String, use_typewriter: bool) {
        let line_count = content.lines().count();
        let force_instant = line_count > 5 || content.len() > 200;

        Self::log_to_file(&content);

        if self.messages.len() >= self.config.max_messages {
            self.messages.remove(0);
            self.cache_dirty = true;
        }

        let typewriter_delay = if use_typewriter && !force_instant {
            self.config.typewriter_delay
        } else {
            Duration::from_millis(0)
        };

        let mut message = Message::new(content, typewriter_delay);
        message.calculate_wrapped_line_count(&self.viewport);
        self.messages.push(message);
        self.cache_dirty = true;
        self.rebuild_line_cache();

        if self.viewport.is_auto_scroll_enabled() {
            let content_height = self.line_cache.len();
            let window_height = self.viewport.window_height();
            if content_height > window_height {
                self.viewport
                    .set_scroll_offset_direct_silent(content_height - window_height);
            }
        }
    }

    pub fn update_typewriter(&mut self) {
        self.persistent_cursor.update_blink();
        if self.config.typewriter_delay.as_millis() == 0 {
            return;
        }

        let mut needs_rebuild = false;
        if let Some(last_message) = self.messages.last_mut() {
            let total_length = last_message.content.graphemes(true).count();
            if let Some(ref mut cursor) = last_message.typewriter_cursor {
                cursor.update_blink();
            }

            if last_message.current_length < total_length {
                let elapsed = last_message.timestamp.elapsed();
                if elapsed >= self.config.typewriter_delay {
                    let old_length = last_message.current_length;
                    let chars_to_add = if self.config.typewriter_delay.as_millis() <= 5 {
                        ((elapsed.as_millis() as f64
                            / self.config.typewriter_delay.as_millis() as f64)
                            .floor()
                            .max(1.0)) as usize
                    } else {
                        1
                    };

                    let new_length = (last_message.current_length + chars_to_add).min(total_length);
                    last_message.current_length = new_length;
                    last_message.timestamp = Instant::now();

                    let next_chars = last_message
                        .content
                        .chars()
                        .skip(old_length)
                        .take(chars_to_add)
                        .collect::<String>();
                    if next_chars.contains('\n') || (new_length - old_length) > 50 {
                        needs_rebuild = true;
                    }

                    self.cache_dirty = true;
                    if new_length == total_length {
                        last_message.typewriter_cursor = None;
                        needs_rebuild = true;
                        self.viewport.enable_auto_scroll_silent();
                        self.viewport.scroll_to_bottom();
                    }
                }
            }
        }
        if needs_rebuild && self.cache_dirty {
            self.rebuild_line_cache();
        }
    }

    pub fn handle_scroll(&mut self, direction: ScrollDirection, amount: usize) {
        match direction {
            ScrollDirection::Up => self.viewport.scroll_up(amount.max(1)),
            ScrollDirection::Down => self.viewport.scroll_down(amount.max(1)),
            ScrollDirection::PageUp => self.viewport.page_up(),
            ScrollDirection::PageDown => self.viewport.page_down(),
            ScrollDirection::ToTop => self.viewport.scroll_to_top(),
            ScrollDirection::ToBottom => self.viewport.scroll_to_bottom(),
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) -> bool {
        let changed = self.viewport.update_terminal_size(width, height);
        if changed {
            for message in &mut self.messages {
                message.calculate_wrapped_line_count(&self.viewport);
            }
            self.cache_dirty = true;
            self.viewport.force_auto_scroll();
        }
        changed
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.line_cache.clear();
        self.cache_dirty = false;
        self.viewport.update_content_height_silent(0);
        self.viewport.force_auto_scroll();
        self.persistent_cursor.show_cursor();
    }

    pub fn create_output_widget_for_rendering(&mut self) -> RenderData<'_> {
        (
            self.get_visible_messages(),
            self.config.clone(),
            self.viewport.output_area(),
            &self.persistent_cursor,
        )
    }

    pub fn update_config(&mut self, new_config: &Config) {
        self.config = new_config.clone();
        self.persistent_cursor = UiCursor::from_config(new_config, CursorKind::Output);
        self.cache_dirty = true;
        if self.messages.len() > self.config.max_messages {
            let excess = self.messages.len() - self.config.max_messages;
            self.messages.drain(0..excess);
            self.cache_dirty = true;
        }
    }

    // Getters
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }
    pub fn get_messages_count(&self) -> usize {
        self.messages.len()
    }
    pub fn get_line_count(&self) -> usize {
        if self.cache_dirty {
            self.messages.iter().map(|m| m.line_count).sum()
        } else {
            self.line_cache.len()
        }
    }
    pub fn get_content_height(&self) -> usize {
        self.viewport.content_height()
    }
    pub fn get_window_height(&self) -> usize {
        self.viewport.window_height()
    }

    pub fn debug_scroll_status(&self) -> String {
        format!(
            "Scroll: offset={}, lines={}, window={}, auto={}, msgs={}, cache={}",
            self.viewport.scroll_offset(),
            self.viewport.content_height(),
            self.viewport.window_height(),
            self.viewport.is_auto_scroll_enabled(),
            self.messages.len(),
            self.line_cache.len()
        )
    }

    pub fn handle_viewport_event(&mut self, event: ViewportEvent) -> bool {
        let changed = self.viewport.handle_event(event);
        if changed {
            for message in &mut self.messages {
                message.calculate_wrapped_line_count(&self.viewport);
            }
            self.cache_dirty = true;
        }
        changed
    }

    pub fn log(&mut self, level: &str, message: &str) {
        self.add_message(format!("[{}] {}", level, message));
    }

    fn log_to_file(content: &str) {
        if content.starts_with("__") || content.trim().is_empty() {
            return;
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(base_dir) = exe_path.parent() {
                let log_path = base_dir.join(".rss").join("rush.logs");
                let _ = std::fs::create_dir_all(log_path.parent().unwrap());
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let log_line = format!("[{}] {}\n", timestamp, content);
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                    .and_then(|mut file| {
                        use std::io::Write;
                        file.write_all(log_line.as_bytes())
                    });
            }
        }
    }

    fn log_startup() {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(base_dir) = exe_path.parent() {
                let log_path = base_dir.join(".rss").join("rush.logs");
                let _ = std::fs::create_dir_all(log_path.parent().unwrap());
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let version = crate::core::constants::VERSION;
                let startup_line = format!(
                    "[{}] === Rush Sync Server v{} Started ===\n",
                    timestamp, version
                );
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                    .and_then(|mut file| {
                        use std::io::Write;
                        file.write_all(startup_line.as_bytes())
                    });
            }
        }
    }
}

// Utility functions
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
    let display_text = marker.trim_start_matches('[').trim_end_matches(']');
    AppColor::from_display_text(display_text) // ← NUR EINE ZEILE!
}

pub fn create_output_widget<'a>(
    messages: &'a [(String, usize, bool, bool, bool)],
    layout_area: crate::ui::viewport::LayoutArea,
    config: &'a Config,
    cursor_state: &'a UiCursor,
) -> Paragraph<'a> {
    let max_lines = layout_area.height as usize;
    if max_lines == 0 || layout_area.width == 0 {
        return Paragraph::new(vec![Line::from(vec![Span::raw(get_translation(
            "screen.render.invalid_layout",
            &[],
        ))])])
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        );
    }

    let safe_max_lines = max_lines.min(1000);
    let mut lines = Vec::new();

    if messages.is_empty() {
        return Paragraph::new(vec![Line::from(vec![Span::raw("")]); safe_max_lines])
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
                    break;
                }

                let is_last_line = line_idx == message_lines.len() - 1;
                let visible_chars = if is_last_message && is_last_line {
                    let chars_before_this_line: usize = message_lines
                        .iter()
                        .take(line_idx)
                        .map(|l| l.graphemes(true).count() + 1)
                        .sum();
                    current_length
                        .saturating_sub(chars_before_this_line)
                        .min(line_content.graphemes(true).count())
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

                if is_last_message
                    && is_last_line
                    && ((*is_typing && *msg_cursor_visible)
                        || (!*is_typing && *persistent_cursor_visible))
                {
                    spans.push(cursor_state.create_cursor_span(config));
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
        lines.push(Line::from(vec![Span::raw(get_translation(
            "screen.render.empty_buffer_error",
            &[],
        ))]));
    }

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        )
        .wrap(Wrap { trim: true })
}
