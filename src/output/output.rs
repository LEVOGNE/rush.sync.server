use crate::prelude::*;
use std::str::FromStr;
use strip_ansi_escapes::strip;

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

        // Extrahiere den Level aus dem Text (z.B. "[DEBUG]", "[LANG]" etc.)
        let level = if let Some(start) = text.find('[') {
            if let Some(end) = text[start..].find(']') {
                &text[start + 1..start + end]
            } else {
                ""
            }
        } else {
            ""
        };

        // Nutze die zentrale Farblogik aus color.rs
        let color = if level.is_empty() {
            config.theme.output_text
        } else if ["DEBUG", "INFO", "WARN", "ERROR"].contains(&level) {
            // Wir parsen den Level-String und fallen auf Info zurück wenn es fehlschlägt
            match Level::from_str(level) {
                Ok(log_level) => AppColor::from_log_level(log_level),
                Err(_) => AppColor::from_log_level(Level::Info),
            }
        } else {
            AppColor::from_custom_level(level)
        };

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
