use crate::core::prelude::*;
use crate::ui::color::AppColor;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

fn parse_message_parts(message: &str) -> Vec<(String, bool)> {
    let clean_message = clean_ansi_codes(message);
    let mut parts = Vec::new();
    let mut current_pos = 0;

    while current_pos < clean_message.len() {
        if let Some(marker_start) = clean_message[current_pos..].find('[') {
            let absolute_start = current_pos + marker_start;

            if marker_start > 0 {
                let text_before = clean_message[current_pos..absolute_start].to_string();
                if !text_before.trim().is_empty() {
                    parts.push((text_before, false));
                }
            }

            if let Some(marker_end) = clean_message[absolute_start..].find(']') {
                let absolute_end = absolute_start + marker_end + 1;
                let marker = clean_message[absolute_start..absolute_end].to_string();
                parts.push((marker, true));
                current_pos = absolute_end;
            } else {
                let remaining = clean_message[absolute_start..].to_string();
                parts.push((remaining, false));
                break;
            }
        } else {
            let remaining = clean_message[current_pos..].to_string();
            if !remaining.trim().is_empty() {
                parts.push((remaining, false));
            }
            break;
        }
    }

    if parts.is_empty() {
        parts.push((clean_message, false));
    }

    parts
}

// ✅ SMART & SKALIERBAR: Dynamisches Mapping mit Fallback
fn get_marker_color(marker: &str) -> AppColor {
    let display_category = marker
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_lowercase();

    if let Some(cat) = display_category.strip_prefix("cat:") {
        let color_category = crate::i18n::get_color_category_for_display(cat);
        return AppColor::from_category_str(&color_category);
    }

    // ✅ SMART: Verwende das erweiterte i18n-System
    let color_category = crate::i18n::get_color_category_for_display(&display_category);
    AppColor::from_category_str(&color_category)
}

fn clean_ansi_codes(message: &str) -> String {
    String::from_utf8_lossy(&strip(message.as_bytes()).unwrap_or_default()).into_owned()
}

fn clean_message_for_display(message: &str) -> String {
    let mut clean = clean_ansi_codes(message);

    if clean.starts_with("__CONFIRM_EXIT__") {
        clean = clean.replace("__CONFIRM_EXIT__", "");
    }
    if clean.starts_with("__CLEAR__") {
        clean = clean.replace("__CLEAR__", "");
    }

    clean.trim().to_string()
}

pub fn create_output_widget<'a>(
    messages: &'a [(&'a String, usize)],
    available_height: u16,
    config: &Config,
) -> Paragraph<'a> {
    let mut lines = Vec::new();
    let max_visible_messages = (available_height as usize).saturating_sub(1);

    if messages.is_empty() {
        let empty_lines = vec![Line::from(vec![Span::raw("")]); max_visible_messages];
        return Paragraph::new(empty_lines)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(config.theme.output_bg.into())),
            )
            .wrap(Wrap { trim: true });
    }

    let start_idx = if messages.len() > max_visible_messages {
        messages.len() - max_visible_messages
    } else {
        0
    };
    let visible_messages = &messages[start_idx..];

    for (idx, (message, current_length)) in visible_messages.iter().enumerate() {
        let is_last_message = idx == visible_messages.len() - 1;
        let clean_message = clean_message_for_display(message);
        let message_parts = parse_message_parts(&clean_message);

        let mut styled_parts = Vec::new();
        let mut total_chars = 0;

        for (part_text, is_marker) in message_parts {
            let part_chars = part_text.graphemes(true).count();
            let part_style = if is_marker {
                Style::default().fg(get_marker_color(&part_text).into())
            } else {
                Style::default().fg(config.theme.output_text.into())
            };

            styled_parts.push((part_text, part_style, part_chars));
            total_chars += part_chars;
        }

        let visible_chars = if is_last_message {
            (*current_length).min(total_chars)
        } else {
            total_chars
        };

        let mut spans = Vec::new();
        let mut chars_used = 0;

        for (part_text, part_style, part_char_count) in styled_parts {
            if chars_used >= visible_chars {
                break;
            }

            let chars_needed = visible_chars - chars_used;

            if chars_needed >= part_char_count {
                spans.push(Span::styled(part_text, part_style));
                chars_used += part_char_count;
            } else {
                let graphemes: Vec<&str> = part_text.graphemes(true).collect();
                let partial_text = graphemes
                    .iter()
                    .take(chars_needed)
                    .copied()
                    .collect::<String>();
                spans.push(Span::styled(partial_text, part_style));
                break;
            }
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        lines.push(Line::from(spans));
    }

    let remaining_space = max_visible_messages.saturating_sub(lines.len());
    for _ in 0..remaining_space {
        lines.push(Line::from(vec![Span::raw("")]));
    }

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(config.theme.output_bg.into())),
        )
        .wrap(Wrap { trim: true })
}
