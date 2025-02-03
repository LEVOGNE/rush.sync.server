// ## FILE: ./src/output.rs
use crate::prelude::*;
use strip_ansi_escapes::strip;

pub fn create_output_widget<'a>(
    messages: &'a [(&'a String, usize)],
    available_height: u16,
    config: &Config,
) -> Paragraph<'a> {
    let mut lines = Vec::new();
    let max_visible_messages = (available_height as usize).saturating_sub(1); // Hier ziehen wir 1 ab

    // Wenn keine Nachrichten da sind, gib ein leeres Widget zurück
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

    // Berechne den korrekten Startindex
    let start_idx = if messages.len() > max_visible_messages {
        messages.len() - max_visible_messages
    } else {
        0
    };
    let visible_messages = &messages[start_idx..];
    let visible_len = visible_messages.len();

    // Verarbeite die sichtbaren Nachrichten
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

        let color = if text.contains("[DEBUG]") {
            Color::Blue
        } else if text.contains("[INFO]") {
            Color::Green
        } else if text.contains("[WARN]") {
            Color::Yellow
        } else if text.contains("[ERROR]") {
            Color::Red
        } else {
            config.theme.output_text.0
        };

        lines.push(Line::from(vec![Span::styled(
            text,
            Style::default().fg(color),
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
