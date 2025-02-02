// ## FILE: ./src/output.rs
use crate::prelude::*;
use strip_ansi_escapes::strip;

pub fn create_output_widget<'a>(
    messages: &'a [(&'a String, usize)],
    available_height: u16,
    config: &Config,
) -> Paragraph<'a> {
    let mut lines = Vec::new();
    let max_visible_messages = available_height as usize;

    // Wenn keine Nachrichten da sind, gib ein leeres Widget zurück
    if messages.is_empty() {
        let empty_lines = vec![Line::from(vec![Span::raw("")]); max_visible_messages];
        return Paragraph::new(empty_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Output")
                    .border_style(Style::default().fg(config.theme.border.into())),
            )
            .wrap(Wrap { trim: true });
    }

    // Bestimme den Startindex für die Anzeige
    // Wichtig: Stelle sicher, dass wir genug Nachrichten für die Anzeige haben
    let start_idx = if messages.len() > max_visible_messages {
        messages.len() - max_visible_messages
    } else {
        0
    };

    // Füge alle sichtbaren Nachrichten hinzu
    for (i, (message, current_length)) in messages.iter().enumerate().skip(start_idx) {
        let text = if i == messages.len() - 1 {
            // Typewriter-Effekt nur für die letzte Nachricht
            let stripped =
                String::from_utf8_lossy(&strip(message).unwrap_or_default()).into_owned();
            let graphemes: Vec<&str> = stripped.graphemes(true).collect();
            graphemes
                .iter()
                .take(*current_length)
                .copied()
                .collect::<String>()
        } else {
            String::from_utf8_lossy(&strip(message).unwrap_or_default()).into_owned()
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

    // Fülle den Rest mit leeren Zeilen auf
    while lines.len() < max_visible_messages {
        lines.push(Line::from(vec![Span::raw("")]));
    }

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Output")
                .border_style(Style::default().fg(config.theme.border.into())),
        )
        .wrap(Wrap { trim: true })
}
