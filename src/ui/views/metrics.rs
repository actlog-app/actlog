use crate::db::models::DaySummary;
use crate::ui::theme::Theme;
use crate::ui::views::summary::format_duration;
use egui::{Color32, RichText, Rounding, Stroke, Ui};

pub fn render_headline_metrics(ui: &mut Ui, summary: Option<&DaySummary>) {
    let theme = Theme::DARK;

    let (active_sec, afk_sec, top_app) = match summary {
        Some(s) => {
            let top = s
                .top_apps
                .first()
                .map(|a| format!("{} ({})", a.app_name, format_duration(a.total_seconds)))
                .unwrap_or_else(|| "None".to_string());
            (s.total_active_seconds, s.total_afk_seconds, top)
        }
        None => (0, 0, "None".to_string()),
    };

    let total = active_sec + afk_sec;
    let avail_width = ui.available_width();
    let col_w = (avail_width - 8.0) / 2.0;

    ui.horizontal(|ui| {
        // Card 1: Total Active Time
        render_metric_box(ui, col_w, |ui| {
            ui.label(RichText::new("TOTAL ACTIVE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(active_sec)).size(24.0).strong().color(theme.text_primary));
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Top Application: {}", top_app)).size(11.0).color(theme.text_muted));
        });

        ui.add_space(8.0);

        // Card 2: Away / Idle Time
        render_metric_box(ui, col_w, |ui| {
            ui.label(RichText::new("AWAY / IDLE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(afk_sec)).size(24.0).strong().color(Color32::from_rgb(255, 152, 0)));
            ui.add_space(4.0);
            let total_str = format_duration(total);
            ui.label(RichText::new(format!("Total Elapsed Today: {}", total_str)).size(11.0).color(theme.text_muted));
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
