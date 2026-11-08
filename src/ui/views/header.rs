use crate::tracker::engine::SharedTrackerState;
use crate::ui::theme::Theme;
use chrono::{Duration, Local, NaiveDate};
use egui::{Color32, RichText, Rounding, Stroke, Ui, Vec2};
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum QuickRange {
    Today,
    Yesterday,
}

pub fn render_header(
    ui: &mut Ui,
    selected_date: &mut NaiveDate,
    quick_range: &mut QuickRange,
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

        // Live Tracker Status & Pause/Resume Control
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
                let toggle_btn = if is_paused { "Resume" } else { "Pause" };
                if ui.button(RichText::new(toggle_btn).size(11.0).color(theme.text_primary)).clicked() {
                    state.is_tracking_paused.fetch_xor(true, Ordering::Relaxed);
                }
            });
        });

        ui.add_space(8.0);

        // Date Navigator (◀ [Today, Nov 04] ▶)
        let frame = egui::Frame::group(ui.style())
            .fill(theme.surface)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

        frame.show(ui, |ui| {
            if ui.button(RichText::new("◀").size(12.0).color(theme.text_muted)).clicked() {
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

            if ui.button(RichText::new("▶").size(12.0).color(theme.text_muted)).clicked() {
                *selected_date += Duration::days(1);
                *on_date_changed = true;
            }
        });

        ui.add_space(8.0);

        // Functional Quick Range Chips: Today & Yesterday
        let ranges = [
            (QuickRange::Today, "Today"),
            (QuickRange::Yesterday, "Yesterday"),
        ];

        for (r, name) in ranges {
            let is_sel = if r == QuickRange::Today {
                *selected_date == today
            } else {
                *selected_date == today - Duration::days(1)
            };

            let bg = if is_sel { theme.accent } else { theme.surface };
            let fg = if is_sel { Color32::WHITE } else { theme.text_muted };

            let btn = ui.add(
                egui::Button::new(RichText::new(name).size(12.0).color(fg))
                    .fill(bg)
                    .rounding(Rounding::same(5.0))
                    .min_size(Vec2::new(56.0, 26.0)),
            );

            if btn.clicked() {
                *quick_range = r;
                match r {
                    QuickRange::Today => *selected_date = today,
                    QuickRange::Yesterday => *selected_date = today - Duration::days(1),
                }
                *on_date_changed = true;
            }
        }
    });
}
