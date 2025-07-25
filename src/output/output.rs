use crate::core::prelude::*;
use crate::i18n;
use crate::ui::color::AppColor;
use log::Level;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

fn extract_log_level(text: &str) -> Option<Level> {
    if let Some(start) = text.find('[') {
        if let Some(end) = text[start..].find(']') {
            let level_str = &text[start + 1..start + end];
            match level_str {
                "DEBUG" => Some(Level::Debug),
                "INFO" => Some(Level::Info),
                "WARN" => Some(Level::Warn),
                "ERROR" => Some(Level::Error),
                "TRACE" => Some(Level::Trace),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

// ✅ EINFACH: Unterstützt BEIDE Marker-Formate ohne Multi-line Komplexität
fn extract_category_and_clean_text(message: &str) -> (Option<String>, String) {
    let stripped = String::from_utf8_lossy(&strip(message).unwrap_or_default()).into_owned();

    // ✅ FORMAT 1: [CAT:category]text
    if let Some(start) = stripped.find("[CAT:") {
        if let Some(end_pos) = stripped[start..].find(']') {
            let end = start + end_pos;
            if end > start + 5 {
                let category = stripped[start + 5..end].to_string();
                let full_marker = &stripped[start..=end];
                let clean_text = stripped.replacen(full_marker, "", 1);
                return (Some(category), clean_prefixes(clean_text));
            }
        }
    }

    // ✅ FORMAT 2: [category]text (für Command-Ausgaben)
    if let Some(start) = stripped.find('[') {
        if let Some(end_pos) = stripped[start..].find(']') {
            let end = start + end_pos;
            if end > start + 1 {
                let category = stripped[start + 1..end].to_string();

                // ✅ Nur bekannte Categories akzeptieren
                if is_known_category(&category) {
                    let full_marker = &stripped[start..=end];
                    let clean_text = stripped.replacen(full_marker, "", 1);
                    return (Some(category), clean_prefixes(clean_text));
                }
            }
        }
    }

    (None, clean_prefixes(stripped))
}

// ✅ HILFSFUNKTION: Bekannte Categories
fn is_known_category(category: &str) -> bool {
    matches!(
        category.to_lowercase().as_str(),
        "lang" | "version" | "warning" | "error" | "info" | "debug" | "trace"
    )
}

// ✅ HILFSFUNKTION: Prefix-Bereinigung
fn clean_prefixes(text: String) -> String {
    let mut clean_text = text;
    if clean_text.starts_with("__CONFIRM_EXIT__") {
        clean_text = clean_text.replace("__CONFIRM_EXIT__", "");
    }
    if clean_text.starts_with("__CLEAR__") {
        clean_text = clean_text.replace("__CLEAR__", "");
    }
    clean_text.trim().to_string()
}

fn get_message_color(message: &str, config: &Config) -> AppColor {
    let (category_opt, clean_text) = extract_category_and_clean_text(message);

    // 1. Use category marker if present
    if let Some(category) = category_opt {
        return AppColor::from_category_str(&category);
    }

    // 2. Check for unknown commands
    if message.starts_with(&i18n::get_translation("system.commands.unknown", &[""])) {
        return i18n::get_translation_color("system.commands.unknown");
    }

    // 3. Check standard log levels
    if let Some(level) = extract_log_level(&clean_text) {
        return AppColor::from_log_level(level);
    }

    // 4. Fallback: theme color
    config.theme.output_text
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
    let visible_len = visible_messages.len();

    for (idx, (message, current_length)) in visible_messages.iter().enumerate() {
        let is_last_message = idx == visible_len - 1;
        let (_, clean_text) = extract_category_and_clean_text(message);

        let text = if is_last_message {
            let graphemes: Vec<&str> = clean_text.graphemes(true).collect();
            graphemes
                .iter()
                .take(*current_length)
                .copied()
                .collect::<String>()
        } else {
            clean_text
        };

        let color = get_message_color(message, config);

        lines.push(Line::from(vec![Span::styled(
            text,
            Style::default().fg(color.into()),
        )]));
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
