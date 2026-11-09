use crate::model::Tag;
use iced::Color;

pub fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn format_pt_time(seconds: i64) -> String {
    let secs = seconds.abs() % 60;
    let mins = (seconds.abs() / 60) % 60;
    let hours = seconds.abs() / 3600;

    format!("{}:{:02}:{:02}", hours, mins, secs)
}

pub fn parse_hex_color(hex: &str) -> Color {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Color::from_rgb(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            );
        }
    }
    Color::from_rgb(100.0 / 255.0, 116.0 / 255.0, 139.0 / 255.0)
}

pub fn resolve_tag_color(tag_id: Option<i64>, tags: &[Tag]) -> Color {
    if let Some(id) = tag_id {
        if let Some(t) = tags.iter().find(|t| t.id == Some(id)) {
            return parse_hex_color(&t.color);
        }
    }
    Color::from_rgb(208.0 / 255.0, 208.0 / 255.0, 208.0 / 255.0) // UNTAGGED default (#D0D0D0)
}
