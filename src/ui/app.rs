use crate::db::models::{ActivitySession, DaySummary, Tag};
use crate::db::repository::Repository;
use crate::tracker::engine::SharedTrackerState;
use crate::ui::views::summary::{format_duration, render_date_bar, render_summary_cards};
use crate::ui::views::table::render_activity_breakdown;
use crate::ui::views::timeline::render_timeline;
use chrono::{Local, NaiveDate, TimeZone};
use eframe::egui::{self, Color32, RichText, Rounding, Ui};
use rusqlite::Connection;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

pub struct ActLogApp {
    state: Arc<SharedTrackerState>,
    db_path: String,
    selected_date: NaiveDate,
    day_summary: Option<DaySummary>,
    day_activities: Vec<ActivitySession>,
    tags: Vec<Tag>,
    last_refresh: Instant,
}

impl ActLogApp {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String) -> Self {
        let mut app = Self {
            state,
            db_path,
            selected_date: Local::now().date_naive(),
            day_summary: None,
            day_activities: Vec::new(),
            tags: Vec::new(),
            last_refresh: Instant::now(),
        };
        app.reload_data();
        app
    }

    pub fn reload_data(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            self.tags = Repository::get_all_tags(&conn).unwrap_or_default();

            let date_str = self.selected_date.format("%Y-%m-%d").to_string();
            let start_of_day = Local
                .from_local_datetime(&self.selected_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap()
                .timestamp();
            let end_of_day = start_of_day + 86400;

            self.day_summary = Repository::get_day_summary(&conn, &date_str, start_of_day, end_of_day).ok();
            self.day_activities = Repository::get_day_activities(&conn, start_of_day, end_of_day).unwrap_or_default();
        }
        self.last_refresh = Instant::now();
    }
}

impl eframe::App for ActLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        static FIRST_FRAME: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if FIRST_FRAME.swap(false, std::sync::atomic::Ordering::Relaxed) {
            println!("ActLog GUI: First frame rendered successfully!");
        }

        // Auto-refresh data every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.reload_data();
        }

        // Repaint every 1000ms so live indicator updates
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));

        egui::CentralPanel::default().show(ctx, |ui| {
            render_top_banner(ui, &self.state);
            ui.add_space(8.0);

            let mut date_changed = false;
            render_date_bar(ui, &mut self.selected_date, &mut date_changed);
            if date_changed {
                self.reload_data();
            }

            ui.add_space(8.0);
            render_summary_cards(ui, self.day_summary.as_ref());

            ui.add_space(8.0);
            let day_start = Local
                .from_local_datetime(&self.selected_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap()
                .timestamp();
            render_timeline(ui, &self.day_activities, &self.tags, day_start);

            ui.add_space(10.0);
            render_activity_breakdown(ui, &self.day_activities, self.day_summary.as_ref(), &self.tags);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    let paused = self.state.is_tracking_paused.load(Ordering::Relaxed);
                    let pause_btn_text = if paused { "▶ Resume Tracking" } else { "⏸ Pause Tracking" };
                    if ui.button(pause_btn_text).clicked() {
                        self.state.is_tracking_paused.store(!paused, Ordering::Relaxed);
                    }
                });
                ui.separator();
            });
        });
    }
}

fn render_top_banner(ui: &mut Ui, state: &Arc<SharedTrackerState>) {
    let frame = egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(22, 24, 29))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let paused = state.is_tracking_paused.load(Ordering::Relaxed);
            let (status_color, status_text) = if paused {
                (Color32::from_rgb(244, 67, 54), "PAUSED")
            } else {
                (Color32::from_rgb(76, 175, 80), "TRACKING ACTIVE")
            };

            let (dot_rect, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, status_color);

            ui.label(RichText::new(status_text).size(12.0).strong().color(status_color));
            ui.separator();

            let current = {
                let lock = state.current_activity.lock().unwrap();
                lock.clone()
            };

            if let Some(act) = current {
                let label = if let Some(ref domain) = act.domain {
                    format!("{}: {} ({})", domain, act.window_title, format_duration(act.duration))
                } else {
                    format!("{}: {} ({})", act.app_name, act.window_title, format_duration(act.duration))
                };
                ui.label(RichText::new(label).strong().size(13.0));
            } else {
                ui.label(RichText::new("Awaiting active window...").color(Color32::GRAY));
            }
        });
    });
}
