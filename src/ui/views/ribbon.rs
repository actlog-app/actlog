use crate::model::{ActivitySession, Tag};
use crate::ui::common::{format_duration, resolve_tag_color};
use crate::ui::theme::Theme;
use chrono::{Local, TimeZone};
use egui::{Color32, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub fn render_day_ribbon(
    ui: &mut Ui,
    activities: &[ActivitySession],
    tags: &[Tag],
    day_start_ts: i64,
) {
    let theme = Theme::DARK;
    let ribbon_height = 26.0;

    let frame = egui::Frame::group(ui.style())
        .fill(theme.surface)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.horizontal(|ui| {
            ui.label(RichText::new("DAY FLOW").size(10.5).strong().color(theme.text_dim));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("24-Hour Chronological Ribbon").size(10.5).color(theme.text_dim));
            });
        });

        ui.add_space(6.0);

        // Ribbon Canvas fills exact inner width
        let ribbon_width = ui.available_width();
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(ribbon_width, ribbon_height), egui::Sense::hover());

        let painter = ui.painter();
        // Background track
        painter.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(22, 25, 32));
        painter.rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, theme.border));

        let day_seconds = 86400.0f32;
        let mut hovered_segment: Option<(String, String, String)> = None;

        for act in activities {
            if act.is_afk {
                continue;
            }

            let start_rel = (act.start_time - day_start_ts).clamp(0, 86400) as f32;
            let end_rel = (act.end_time - day_start_ts).clamp(0, 86400) as f32;

            if end_rel <= start_rel {
                continue;
            }

            let x_start = rect.left() + (start_rel / day_seconds) * rect.width();
            let x_end = (rect.left() + (end_rel / day_seconds) * rect.width()).max(x_start + 2.0);

            let seg_rect = Rect::from_min_max(
                Pos2::new(x_start, rect.top() + 2.0),
                Pos2::new(x_end, rect.bottom() - 2.0),
            );

            let color = resolve_tag_color(act.tag_id, tags);
            painter.rect_filled(seg_rect, Rounding::same(2.0), color);

            // Hover check
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                if seg_rect.contains(mouse_pos) {
                    let start_dt = Local.timestamp_opt(act.start_time, 0).single();
                    let end_dt = Local.timestamp_opt(act.end_time, 0).single();
                    let time_range = match (start_dt, end_dt) {
                        (Some(s), Some(e)) => format!("{} - {}", s.format("%H:%M"), e.format("%H:%M")),
                        _ => format_duration(act.duration),
                    };
                    hovered_segment = Some((act.app_name.clone(), act.window_title.clone(), time_range));
                }
            }
        }

        // Time ticks below ribbon
        ui.add_space(4.0);
        let ticks = ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00", "24:00"];
        let tick_w = (rect.width() - 40.0) / (ticks.len() - 1) as f32;

        ui.horizontal(|ui| {
            ui.add_space(2.0);
            for (i, t) in ticks.iter().enumerate() {
                ui.label(RichText::new(*t).size(9.5).color(theme.text_dim));
                if i < ticks.len() - 1 {
                    ui.add_space(tick_w - 22.0);
                }
            }
        });

        // Hover Tooltip Display
        if let Some((app, title, time_span)) = hovered_segment {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Hovered:").size(11.0).color(theme.accent_hover).strong());
                ui.label(RichText::new(format!("{} • {} ({})", app, title, time_span)).size(11.0).color(theme.text_primary));
            });
        }
    });
}
