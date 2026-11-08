use crate::tracker::SharedTrackerState;
use crate::ui::theme::Theme;
use chrono::{Duration, Local, NaiveDate};
use egui::{Color32, RichText, Rounding, Stroke, Ui, Vec2};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub fn render_header(
    ui: &mut Ui,
    selected_date: &mut NaiveDate,
    state: &Arc<SharedTrackerState>,
    on_date_changed: &mut bool,
) {
    let theme = Theme::DARK;
    let today = Local::now().date_naive();
    let is_paused = state.is_tracking_paused.load(Ordering::Relaxed);

    ui.horizontal(|ui| {
        // App Brand & Version Badge
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("ActLog")
                    .strong()
                    .size(15.0)
                    .color(theme.text_primary),
            );
            let ver_badge = egui::Frame::group(ui.style())
                .fill(theme.surface)
                .rounding(Rounding::same(4.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::symmetric(5.0, 2.0));
            ver_badge.show(ui, |ui| {
                ui.label(
                    RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .size(11.0)
                        .color(theme.accent),
                );
            });
        });

        ui.add_space(8.0);

        // Live Tracker Status & Seamless Pause/Resume Control
        let (status_text, status_color) = if is_paused {
            ("⏸ Paused", Color32::from_rgb(245, 158, 11))
        } else {
            ("● Live", Color32::from_rgb(34, 197, 94))
        };

        let status_frame = egui::Frame::group(ui.style())
            .fill(theme.surface)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::symmetric(8.0, 3.0));

        status_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(status_text).size(11.5).strong().color(status_color));
                ui.add_space(2.0);
                let toggle_label = if is_paused { "Resume" } else { "Pause" };
                let toggle_btn = ui.add(
                    egui::Button::new(RichText::new(toggle_label).size(11.0).color(theme.text_muted))
                        .frame(false),
                );
                if toggle_btn.clicked() {
                    state.is_tracking_paused.fetch_xor(true, Ordering::Relaxed);
                }
            });
        });

        ui.add_space(8.0);

        // Date Navigator (◀ [Today, Nov 08] ▶) with bounded future navigation
        let can_go_forward = *selected_date < today;
        let frame = egui::Frame::group(ui.style())
            .fill(theme.surface)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

        frame.show(ui, |ui| {
            let back_btn = ui.add(
                egui::Button::new(RichText::new("◀").size(12.0).color(theme.text_muted))
                    .frame(false),
            );
            if back_btn.clicked() {
                *selected_date -= Duration::days(1);
                *on_date_changed = true;
            }

            let label = if *selected_date == today {
                format!("Today, {}", selected_date.format("%b %d"))
            } else if *selected_date == today - Duration::days(1) {
                format!("Yesterday, {}", selected_date.format("%b %d"))
            } else {
                selected_date.format("%a, %b %d").to_string()
            };

            ui.label(RichText::new(label).strong().size(13.0).color(theme.text_primary));

            let fwd_color = if can_go_forward { theme.text_muted } else { theme.border };
            let fwd_btn = ui.add_enabled(
                can_go_forward,
                egui::Button::new(RichText::new("▶").size(12.0).color(fwd_color)).frame(false),
            );
            if fwd_btn.clicked() {
                *selected_date += Duration::days(1);
                *on_date_changed = true;
            }
        });

        ui.add_space(8.0);

        // Quick Date Chips: Today & Yesterday (dynamically derived from selected_date)
        let chips = [
            ("Today", today),
            ("Yesterday", today - Duration::days(1)),
        ];

        for (name, chip_date) in chips {
            let is_sel = *selected_date == chip_date;
            let bg = if is_sel { theme.accent } else { theme.surface };
            let fg = if is_sel { Color32::WHITE } else { theme.text_muted };

            let btn = ui.add(
                egui::Button::new(RichText::new(name).size(12.0).color(fg))
                    .fill(bg)
                    .rounding(Rounding::same(5.0))
                    .min_size(Vec2::new(56.0, 26.0)),
            );

            if btn.clicked() {
                *selected_date = chip_date;
                *on_date_changed = true;
            }
        }
    });
}
