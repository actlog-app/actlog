use crate::db::models::DaySummary;
use chrono::{Duration, Local, NaiveDate};
use egui::{Color32, RichText, Rounding, Ui, Vec2};

pub fn render_date_bar(ui: &mut Ui, selected_date: &mut NaiveDate, on_change: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("◀ Prev Day").clicked() {
            *selected_date -= Duration::days(1);
            *on_change = true;
        }

        let is_today = *selected_date == Local::now().date_naive();
        let date_label = if is_today {
            format!("Today ({})", selected_date.format("%Y-%m-%d"))
        } else {
            selected_date.format("%Y-%m-%d").to_string()
        };

        ui.label(RichText::new(date_label).strong().size(16.0));

        if ui.button("Next Day ▶").clicked() {
            *selected_date += Duration::days(1);
            *on_change = true;
        }

        if !is_today {
            if ui.button("Go to Today").clicked() {
                *selected_date = Local::now().date_naive();
                *on_change = true;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🔄 Refresh").clicked() {
                *on_change = true;
            }
        });
    });
}

pub fn render_summary_cards(ui: &mut Ui, summary: Option<&DaySummary>) {
    let (active_sec, afk_sec) = match summary {
        Some(s) => (s.total_active_seconds, s.total_afk_seconds),
        None => (0, 0),
    };

    ui.horizontal(|ui| {
        render_metric_card(
            ui,
            "ACTIVE TIME",
            &format_duration(active_sec),
            Color32::from_rgb(76, 175, 80),
        );
        ui.add_space(8.0);
        render_metric_card(
            ui,
            "IDLE / AWAY",
            &format_duration(afk_sec),
            Color32::from_rgb(255, 152, 0),
        );
        ui.add_space(8.0);

        let total = active_sec + afk_sec;
        let score_text = if total > 0 {
            let focus_ratio = (active_sec as f64 / total as f64) * 100.0;
            format!("{:.0}% Focus", focus_ratio)
        } else {
            "0% Focus".to_string()
        };

        render_metric_card(
            ui,
            "PRODUCTIVITY",
            &score_text,
            Color32::from_rgb(33, 150, 243),
        );
    });
}

fn render_metric_card(ui: &mut Ui, title: &str, value: &str, accent_color: Color32) {
    let frame = egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(28, 30, 36))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0));

    frame.show(ui, |ui| {
        ui.set_min_size(Vec2::new(140.0, 60.0));
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(11.0).color(Color32::from_rgb(140, 140, 140)));
            ui.add_space(2.0);
            ui.label(RichText::new(value).size(20.0).strong().color(accent_color));
        });
    });
}

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
