use crate::core::prelude::*;
use crate::i18n;
use crate::ui::color::AppColor;
use log::Level;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::str::FromStr;
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

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
        let stripped = String::from_utf8_lossy(&strip(message).unwrap_or_default()).into_owned();

        let text = if is_last_message {
            let graphemes: Vec<&str> = stripped.graphemes(true).collect();
            graphemes
                .iter()
                .take(*current_length)
                .copied()
                .collect::<String>()
        } else {
            stripped
        };

        let (level_text, color) =
            if message.starts_with(&i18n::get_translation("system.commands.unknown", &[""])) {
                let (_, category) = i18n::get_translation_details("system.commands.unknown");
                // Für unbekannte Befehle holen wir uns den Fehler-Level-Text
                let (error_text, _) = i18n::get_translation_details("system.log.error");
                (
                    format!("[{}] ", error_text),
                    AppColor::from_category(category),
                )
            } else {
                // Extrahiere den Level aus dem Text
                if let Some(start) = text.find('[') {
                    if let Some(end) = text[start..].find(']') {
                        let level = &text[start + 1..start + end];
                        if ["DEBUG", "INFO", "WARN", "ERROR"].contains(&level) {
                            match Level::from_str(level) {
                                Ok(log_level) => (
                                    format!(
                                        "[{}] ",
                                        i18n::get_translation(
                                            &format!("system.log.{}", level.to_lowercase()),
                                            &[]
                                        )
                                    ),
                                    AppColor::from_log_level(log_level),
                                ),
                                Err(_) => ("".to_string(), AppColor::from_log_level(Level::Info)),
                            }
                        } else {
                            ("".to_string(), AppColor::from_custom_level(level, None))
                        }
                    } else {
                        ("".to_string(), config.theme.output_text)
                    }
                } else {
                    ("".to_string(), config.theme.output_text)
                }
            };

        // Text zusammenbauen, dabei Level-Text voranstellen wenn vorhanden
        let display_text = if !level_text.is_empty() {
            format!("{}{}", level_text, text)
        } else {
            text
        };

        lines.push(Line::from(vec![Span::styled(
            display_text,
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
