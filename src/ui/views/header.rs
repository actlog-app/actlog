use crate::tracker::engine::SharedTrackerState;
use crate::ui::theme::Theme;
use crate::ui::views::summary::format_duration;
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

    ui.horizontal(|ui| {
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

        // Right side: Live tracker badge and Pause/Resume toggle
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let paused = state.is_tracking_paused.load(Ordering::Relaxed);
            let pause_btn = if paused {
                egui::Button::new(RichText::new("▶ Resume").size(12.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(76, 175, 80))
            } else {
                egui::Button::new(RichText::new("⏸ Pause").size(12.0).color(theme.text_muted))
                    .fill(theme.surface)
            };

            if ui.add(pause_btn.rounding(Rounding::same(5.0))).clicked() {
                state.is_tracking_paused.store(!paused, Ordering::Relaxed);
            }

            ui.add_space(8.0);

            // Live Activity Pill
            let live_card = egui::Frame::group(ui.style())
                .fill(theme.surface)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::symmetric(10.0, 5.0));

            live_card.show(ui, |ui| {
                let dot_color = if paused {
                    Color32::from_rgb(244, 67, 54)
                } else {
                    Color32::from_rgb(76, 175, 80)
                };

                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);

                ui.add_space(4.0);

                let current = state.current_activity.lock().unwrap().clone();
                let status_text = if paused {
                    "Tracking Paused".to_string()
                } else if let Some(act) = current {
                    if act.app_name.eq_ignore_ascii_case("actlog.exe") {
                        "Viewing Dashboard".to_string()
                    } else {
                        format!("{}: {} ({})", act.app_name, truncate(&act.window_title, 20), format_duration(act.duration))
                    }
                } else {
                    "Monitoring desktop...".to_string()
                };

                ui.label(RichText::new(status_text).size(12.0).color(theme.text_primary));
            });
        });
    });
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    }
}
