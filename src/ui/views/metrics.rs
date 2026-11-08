use crate::model::DaySummary;
use crate::ui::common::format_duration;
use crate::ui::theme::Theme;
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

    ui.columns(2, |cols| {
        // Card 1: Total Active Time
        let frame1 = egui::Frame::group(cols[0].style())
            .fill(theme.surface)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::symmetric(14.0, 10.0));

        frame1.show(&mut cols[0], |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("TOTAL ACTIVE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(active_sec)).size(24.0).strong().color(theme.text_primary));
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Top Application: {}", top_app)).size(11.0).color(theme.text_muted));
        });

        // Card 2: Away / Idle Time
        let frame2 = egui::Frame::group(cols[1].style())
            .fill(theme.surface)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::symmetric(14.0, 10.0));

        frame2.show(&mut cols[1], |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("AWAY / IDLE TIME").size(10.5).strong().color(theme.text_dim));
            ui.add_space(2.0);
            ui.label(RichText::new(format_duration(afk_sec)).size(24.0).strong().color(Color32::from_rgb(255, 152, 0)));
            ui.add_space(4.0);
            let total_str = format_duration(total);
            ui.label(RichText::new(format!("Total Elapsed Today: {}", total_str)).size(11.0).color(theme.text_muted));
        });
    });
}
