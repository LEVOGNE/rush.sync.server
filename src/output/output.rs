// ## BEGIN ##
use crate::core::prelude::*;
use crate::ui::color::AppColor;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

/// ✅ Entfernt ANSI-Codes aus Logs (z.B. falls doch welche reinkommen)
fn clean_ansi_codes(message: &str) -> String {
    String::from_utf8_lossy(&strip(message.as_bytes()).unwrap_or_default()).into_owned()
}

/// ✅ Entfernt interne Steuerzeichen wie __CLEAR__
fn clean_message_for_display(message: &str) -> String {
    clean_ansi_codes(message)
        .replace("__CONFIRM_EXIT__", "")
        .replace("__CLEAR__", "")
        .trim()
        .to_string()
}

/// ✅ Teilt Message in Text + Marker ([INFO], [ERROR], etc.)
// ## FILE: src/output/output.rs (Optimiert) ##
fn parse_message_parts(message: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut chars = message.char_indices().peekable();
    let mut start = 0;

    while let Some((i, c)) = chars.peek().cloned() {
        if c == '[' {
            // Text vor dem Marker
            if start < i {
                let text = &message[start..i];
                if !text.trim().is_empty() {
                    parts.push((text.to_owned(), false));
                }
            }

            // Marker finden
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

/// ✅ Marker-Farben: nutzt i18n → z.B. "info" → AppColor
fn get_marker_color(marker: &str) -> AppColor {
    let display_category = marker
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_start_matches("cat:")
        .to_lowercase();

    // ✅ 1. Standard-Keys direkt
    if AppColor::from_any(&display_category).to_name() != "gray" {
        return AppColor::from_any(&display_category);
    }

    // ✅ 2. Übersetzte Marker → i18n-Mapping
    let mapped_category = crate::i18n::get_color_category_for_display(&display_category);
    AppColor::from_any(mapped_category)
}

/// ✅ Hauptfunktion: Baut den fertigen Paragraph
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

    let start_idx = messages.len().saturating_sub(max_visible_messages);
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
// ## END ##
