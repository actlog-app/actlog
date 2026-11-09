use crate::model::Tag;
use egui::{Color32, FontId, Margin, Pos2, Rect, Rounding, Stroke, Ui, Vec2};

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
    Color32::from_rgb(100, 116, 139)
}

pub fn resolve_tag_color(tag_id: Option<i64>, tags: &[Tag]) -> Color32 {
    if let Some(id) = tag_id {
        if let Some(t) = tags.iter().find(|t| t.id == Some(id)) {
            return parse_hex_color(&t.color);
        }
    }
    Color32::from_rgb(208, 208, 208) // UNTAGGED default in PT (#D0D0D0)
}

/// 1:1 Win32 Etched GroupBox with embedded title cutout in top border line
pub fn render_win32_groupbox<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let frame = egui::Frame::none()
        .stroke(Stroke::new(1.0, Color32::from_rgb(208, 208, 208)))
        .rounding(Rounding::ZERO)
        .inner_margin(Margin {
            left: 5.0,
            right: 5.0,
            top: 7.0,
            bottom: 3.0,
        });

    ui.add_space(3.0);
    let resp = frame.show(ui, |ui| add_contents(ui));
    let rect = resp.response.rect;

    // Draw title text interrupting top line of groupbox
    let text_pos = Pos2::new(rect.left() + 8.0, rect.top() - 6.0);
    let painter = ui.painter();
    let galley = painter.layout_no_wrap(
        title.to_string(),
        FontId::proportional(11.0),
        Color32::from_rgb(0, 0, 0),
    );
    let text_rect = Rect::from_min_size(text_pos, galley.size()).expand(2.0);
    // Erase top border segment with dialog background
    painter.rect_filled(text_rect, Rounding::ZERO, Color32::from_rgb(240, 240, 240));
    painter.galley(text_pos, galley, Color32::from_rgb(0, 0, 0));

    resp.inner
}

/// 1:1 Win32 square [+] / [-] tree expander glyph
pub fn render_win32_tree_toggle(ui: &mut Ui, is_expanded: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(9.0, 9.0), egui::Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::ZERO, Color32::from_rgb(255, 255, 255));
    painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::from_rgb(128, 128, 128)));

    // Minus horizontal bar
    let mid_y = rect.center().y;
    painter.line_segment(
        [Pos2::new(rect.left() + 2.0, mid_y), Pos2::new(rect.right() - 2.0, mid_y)],
        Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
    );

    // Plus vertical bar (if collapsed)
    if !is_expanded {
        let mid_x = rect.center().x;
        painter.line_segment(
            [Pos2::new(mid_x, rect.top() + 2.0), Pos2::new(mid_x, rect.bottom() - 2.0)],
            Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
        );
    }

    resp.clicked()
}

/// Draws a dotted vertical guideline for tree hierarchies (1:1 with Win32 TVS_HASLINES)
pub fn draw_dotted_v_line(painter: &egui::Painter, x: f32, y1: f32, y2: f32, color: Color32) {
    let mut y = y1.floor();
    let end_y = y2.floor();
    while y <= end_y {
        painter.line_segment(
            [Pos2::new(x, y), Pos2::new(x, (y + 1.0).min(end_y))],
            Stroke::new(1.0, color),
        );
        y += 2.0;
    }
}

/// Draws a dotted horizontal guideline connecting parent to child node
pub fn draw_dotted_h_line(painter: &egui::Painter, x1: f32, x2: f32, y: f32, color: Color32) {
    let mut x = x1.floor();
    let end_x = x2.floor();
    while x <= end_x {
        painter.line_segment(
            [Pos2::new(x, y), Pos2::new((x + 1.0).min(end_x), y)],
            Stroke::new(1.0, color),
        );
        x += 2.0;
    }
}
