// output/output.rs - KOMPLETT NEU UND SAUBER
use crate::core::prelude::*;
use crate::ui::color::AppColor;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use strip_ansi_escapes::strip;
use unicode_segmentation::UnicodeSegmentation;

/// ✅ HAUPTFUNKTION: Bestimmt Farbe basierend auf Message-Inhalt
fn get_message_color(message: &str, config: &Config) -> AppColor {
    let clean_message = clean_ansi_codes(message);

    // 1. Command-Category Marker: [category] (alle Categories, dann Mapping)
    if let Some(display_category) = extract_any_category_marker(&clean_message) {
        // ✅ Hole color_category aus JSON-Mapping
        let color_category = crate::i18n::get_color_category_for_display(&display_category);
        return AppColor::from_category_str(&color_category);
    }

    // 2. Standard Log-Level: [DEBUG], [INFO], [WARN], [ERROR], [TRACE]
    if let Some(level_str) = extract_log_level_marker(&clean_message) {
        return match level_str.as_str() {
            "DEBUG" => AppColor::from_category_str("debug"),
            "INFO" => AppColor::from_category_str("info"),
            "WARN" => AppColor::from_category_str("warning"),
            "ERROR" => AppColor::from_category_str("error"),
            "TRACE" => AppColor::from_category_str("trace"),
            _ => config.theme.output_text,
        };
    }

    // 3. Spezielle Nachrichten
    if clean_message.contains("Translation key not found") {
        return AppColor::from_category_str("warning");
    }

    if clean_message.contains("Unknown command") || clean_message.contains("Unbekannter Befehl") {
        return AppColor::from_category_str("error");
    }

    // 4. Fallback: Theme-Standard
    config.theme.output_text
}

/// ✅ HILFSFUNKTION: ANSI-Codes entfernen
fn clean_ansi_codes(message: &str) -> String {
    String::from_utf8_lossy(&strip(message.as_bytes()).unwrap_or_default()).into_owned()
}

/// ✅ HILFSFUNKTION: Alle Category-Marker extrahieren (für JSON-Mapping)
fn extract_any_category_marker(message: &str) -> Option<String> {
    // Format: [CAT:category]
    if let Some(start) = message.find("[CAT:") {
        if let Some(end) = message[start..].find(']') {
            let category = &message[start + 5..start + end];
            return Some(category.to_lowercase());
        }
    }

    // Format: [category] - alle Categories akzeptieren
    if let Some(start) = message.find('[') {
        if let Some(end) = message[start..].find(']') {
            let potential_category = &message[start + 1..start + end];

            // ✅ Ignoriere nur Standard-Log-Level (die werden separat behandelt)
            let upper_cat = potential_category.to_uppercase();
            if !matches!(
                upper_cat.as_str(),
                "DEBUG" | "INFO" | "WARN" | "ERROR" | "TRACE"
            ) {
                return Some(potential_category.to_lowercase());
            }
        }
    }

    None
}

/// ✅ HILFSFUNKTION: Log-Level Marker extrahieren
fn extract_log_level_marker(message: &str) -> Option<String> {
    if let Some(start) = message.find('[') {
        if let Some(end) = message[start..].find(']') {
            let level_str = &message[start + 1..start + end];

            // Standard-Log-Level akzeptieren (case-insensitive, return uppercase)
            if matches!(
                level_str.to_uppercase().as_str(),
                "DEBUG" | "INFO" | "WARN" | "ERROR" | "TRACE"
            ) {
                return Some(level_str.to_uppercase());
            }
        }
    }

    None
}

/// ✅ HILFSFUNKTION: Message für Anzeige bereinigen
fn clean_message_for_display(message: &str) -> String {
    let mut clean = clean_ansi_codes(message);

    // Entferne spezielle Prefixes
    if clean.starts_with("__CONFIRM_EXIT__") {
        clean = clean.replace("__CONFIRM_EXIT__", "");
    }
    if clean.starts_with("__CLEAR__") {
        clean = clean.replace("__CLEAR__", "");
    }

    clean.trim().to_string()
}

/// ✅ HAUPTFUNKTION: Widget erstellen
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

    // Berechne sichtbare Nachrichten
    let start_idx = if messages.len() > max_visible_messages {
        messages.len() - max_visible_messages
    } else {
        0
    };
    let visible_messages = &messages[start_idx..];

    // Verarbeite jede Nachricht
    for (idx, (message, current_length)) in visible_messages.iter().enumerate() {
        let is_last_message = idx == visible_messages.len() - 1;
        let clean_message = clean_message_for_display(message);

        // Typewriter-Effekt nur für letzte Nachricht
        let display_text = if is_last_message {
            let graphemes: Vec<&str> = clean_message.graphemes(true).collect();
            graphemes
                .iter()
                .take(*current_length)
                .copied()
                .collect::<String>()
        } else {
            clean_message
        };

        // Bestimme Farbe basierend auf Original-Message
        let color = get_message_color(message, config);

        lines.push(Line::from(vec![Span::styled(
            display_text,
            Style::default().fg(color.into()),
        )]));
    }

    // Fülle verbleibenden Platz
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
