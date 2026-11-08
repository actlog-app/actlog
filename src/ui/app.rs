use crate::model::{ActivitySession, DaySummary, Tag};
use crate::db::repository::Repository;
use crate::tracker::SharedTrackerState;
use crate::ui::theme::configure_impeccable_theme;
use crate::ui::views::explorer::render_apps_and_categories;
use crate::ui::views::header::{render_header, QuickRange};
use crate::ui::views::metrics::render_headline_metrics;
use crate::ui::views::ribbon::render_day_ribbon;
use chrono::{Local, NaiveDate, TimeZone};
use eframe::egui;
use rusqlite::Connection;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

pub struct ActLogApp {
    #[allow(dead_code)]
    state: Arc<SharedTrackerState>,
    db_path: String,

    // Header State
    selected_date: NaiveDate,
    quick_range: QuickRange,

    // Filter & Search State
    selected_category: Option<String>,
    search_query: String,
    expanded_apps: HashSet<String>,

    // Data Cache
    activities: Vec<ActivitySession>,
    summary: Option<DaySummary>,
    tags: Vec<Tag>,
    last_refresh: Instant,
    theme_configured: bool,
}

impl ActLogApp {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String) -> Self {
        let today = Local::now().date_naive();
        let mut app = Self {
            state,
            db_path,
            selected_date: today,
            quick_range: QuickRange::Today,
            selected_category: None,
            search_query: String::new(),
            expanded_apps: HashSet::new(),
            activities: Vec::new(),
            summary: None,
            tags: Vec::new(),
            last_refresh: Instant::now(),
            theme_configured: false,
        };
        app.reload_data();
        app
    }

    pub fn reload_data(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            self.tags = Repository::get_all_tags(&conn).unwrap_or_default();

            let (start_ts, end_ts) = self.calculate_timestamps();
            self.activities =
                Repository::get_day_activities(&conn, start_ts, end_ts).unwrap_or_default();

            let date_str = self.selected_date.format("%Y-%m-%d").to_string();
            self.summary =
                Repository::get_day_summary(&conn, &date_str, start_ts, end_ts).ok();
        }
        self.last_refresh = Instant::now();
    }

    fn calculate_timestamps(&self) -> (i64, i64) {
        let s = Local
            .from_local_datetime(&self.selected_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        let next_date = self.selected_date.succ_opt().unwrap_or(self.selected_date);
        let e = Local
            .from_local_datetime(&next_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        (s, e)
    }
}

impl eframe::App for ActLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_configured {
            configure_impeccable_theme(ctx);
            self.theme_configured = true;
        }

        // Auto-refresh data every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.reload_data();
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));

        egui::CentralPanel::default().show(ctx, |ui| {
            // Layer 1: Header (Date navigation + Live pulse status + Pause toggle)
            let mut date_changed = false;
            render_header(
                ui,
                &mut self.selected_date,
                &mut self.quick_range,
                &self.state,
                &mut date_changed,
            );
            if date_changed {
                self.reload_data();
            }

            ui.add_space(10.0);

            // Layer 2: Headline Metrics (Active Time + Away Time)
            render_headline_metrics(ui, self.summary.as_ref());

            ui.add_space(10.0);

            // Layer 3: Day Flow Ribbon (24-hour visual chronological ribbon)
            let day_start = Local
                .from_local_datetime(&self.selected_date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap()
                .timestamp();
            render_day_ribbon(ui, &self.activities, &self.tags, day_start);

            ui.add_space(10.0);

            // Layer 4: Exploration Split View (Categories on left, Search & Apps on right)
            let mut tag_assigned = None;
            render_apps_and_categories(
                ui,
                &self.activities,
                &self.tags,
                &mut self.selected_category,
                &mut self.search_query,
                &mut self.expanded_apps,
                &mut tag_assigned,
            );

            if let Some((app_name, new_tag_id)) = tag_assigned {
                if let Ok(conn) = Connection::open(&self.db_path) {
                    let _ = conn.execute(
                        "UPDATE activities SET tag_id = ?1 WHERE app_name = ?2",
                        (new_tag_id, &app_name),
                    );
                    let _ = conn.execute(
                        "DELETE FROM rules WHERE pattern = ?1 AND match_field = 'app_name'",
                        [&app_name],
                    );
                    let _ = conn.execute(
                        "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, 'app_name', ?2, 20)",
                        (&app_name, new_tag_id),
                    );
                }
                self.reload_data();
            }
        });
    }
}
