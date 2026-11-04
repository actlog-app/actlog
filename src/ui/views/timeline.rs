use crate::db::models::{ActivitySession, Tag};
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, Vec2};

pub fn render_timeline(
    ui: &mut Ui,
    activities: &[ActivitySession],
    tags: &[Tag],
    day_start_ts: i64,
) {
    let desired_size = Vec2::new(ui.available_width(), 32.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    // Draw background track (24h)
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(24, 25, 29));
    painter.rect_stroke(
        rect,
        Rounding::same(4.0),
        Stroke::new(1.0, Color32::from_rgb(45, 48, 55)),
    );

    let day_duration = 86400.0f32; // 24 hours in seconds

    for act in activities {
        if act.is_afk {
            continue;
        }

        let start_rel = (act.start_time - day_start_ts).max(0) as f32;
        let end_rel = (act.end_time - day_start_ts).min(86400) as f32;

        if end_rel <= start_rel {
            continue;
        }

        let x_start = rect.left() + (start_rel / day_duration) * rect.width();
        let x_end = (rect.left() + (end_rel / day_duration) * rect.width()).max(x_start + 2.0);

        let seg_rect = Rect::from_min_max(
            Pos2::new(x_start, rect.top() + 2.0),
            Pos2::new(x_end, rect.bottom() - 2.0),
        );

        let color = resolve_tag_color(act.tag_id, tags);
        painter.rect_filled(seg_rect, Rounding::same(2.0), color);
    }
}

pub fn resolve_tag_color(tag_id: Option<i64>, tags: &[Tag]) -> Color32 {
    if let Some(id) = tag_id {
        if let Some(t) = tags.iter().find(|t| t.id == Some(id)) {
            return parse_hex_color(&t.color);
        }
    }
    Color32::from_rgb(117, 117, 117) // Default grey
}

pub fn parse_hex_color(hex: &str) -> Color32 {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Color32::from_rgb(r, g, b);
        }
    }
    Color32::from_rgb(76, 175, 80)
}
