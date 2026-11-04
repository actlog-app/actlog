use crate::db::models::{DaySummary, Tag};
use crate::ui::theme::Theme;
use crate::ui::views::summary::format_duration;
use egui::{Color32, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub fn render_headline_metrics(ui: &mut Ui, summary: Option<&DaySummary>, tags: &[Tag]) {
    let theme = Theme::DARK;

    let (active_sec, afk_sec, top_app) = match summary {
        Some(s) => {
            let top = s.top_apps.first().map(|a| format!("{} ({})", a.app_name, format_duration(a.total_seconds)))
                .unwrap_or_else(|| "None".to_string());
            (s.total_active_seconds, s.total_afk_seconds, top)
        }
        None => (0, 0, "None".to_string()),
    };

    let total = active_sec + afk_sec;
    let focus_ratio = if total > 0 {
        let work_sec: i64 = summary
            .map(|s| {
                s.categories
                    .iter()
                    .filter(|c| is_focus_tag(&c.tag_name, tags))
                    .map(|c| c.total_seconds)
                    .sum()
            })
            .unwrap_or(0);
        ((work_sec as f64 / active_sec.max(1) as f64) * 100.0).clamp(0.0, 100.0) as i64
    } else {
        0
    };

    let avail_width = ui.available_width();
    let col_w = (avail_width - 16.0) / 3.0;

    ui.horizontal(|ui| {
        // Hero Card 1: Focus Score
        render_metric_box(ui, col_w, |ui| {
            ui.label(RichText::new("FOCUS SCORE").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);

            let score_color = if focus_ratio >= 70 {
                Color32::from_rgb(76, 175, 80) // Green
            } else if focus_ratio >= 40 {
                Color32::from_rgb(255, 152, 0) // Amber
            } else {
                Color32::from_rgb(244, 67, 54) // Red
            };

            ui.label(RichText::new(format!("{}%", focus_ratio)).size(24.0).strong().color(score_color));
            ui.add_space(4.0);

            // Mini Focus Ratio Bar
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(col_w - 24.0, 6.0), egui::Sense::hover());
            ui.painter().rect_filled(bar_rect, Rounding::same(3.0), Color32::from_rgb(32, 36, 45));

            let fill_w = ((col_w - 24.0) * (focus_ratio as f32 / 100.0)).max(2.0);
            let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), score_color);
        });

        ui.add_space(8.0);

        // Hero Card 2: Active Time
        render_metric_box(ui, col_w, |ui| {
            ui.label(RichText::new("TOTAL ACTIVE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(active_sec)).size(24.0).strong().color(theme.text_primary));
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Top: {}", top_app)).size(11.0).color(theme.text_muted));
        });

        ui.add_space(8.0);

        // Hero Card 3: Idle / Away
        render_metric_box(ui, col_w, |ui| {
            ui.label(RichText::new("AWAY / IDLE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(afk_sec)).size(24.0).strong().color(Color32::from_rgb(255, 152, 0)));
            ui.add_space(4.0);
            let total_str = format_duration(total);
            ui.label(RichText::new(format!("Total Logged: {}", total_str)).size(11.0).color(theme.text_muted));
        });
    });
}

fn render_metric_box(ui: &mut Ui, width: f32, add_contents: impl FnOnce(&mut Ui)) {
    let theme = Theme::DARK;
    let frame = egui::Frame::group(ui.style())
        .fill(theme.surface)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

    frame.show(ui, |ui| {
        ui.set_width(width);
        ui.vertical(add_contents);
    });
}

fn is_focus_tag(tag_name: &str, tags: &[Tag]) -> bool {
    if let Some(t) = tags.iter().find(|t| t.name.eq_ignore_ascii_case(tag_name)) {
        t.productivity_score > 0
    } else {
        tag_name.eq_ignore_ascii_case("work") || tag_name.eq_ignore_ascii_case("development")
    }
}

