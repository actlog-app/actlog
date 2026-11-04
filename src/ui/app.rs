use crate::db::models::{ActivitySession, Tag};
use crate::db::repository::Repository;
use crate::tracker::engine::SharedTrackerState;
use crate::ui::views::timeline::parse_hex_color;
use crate::ui::views::tree::{format_pt_time, render_procrastitracker_tree, ActivityNode};
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use eframe::egui::{self, Color32, Rounding, Stroke, Ui, Vec2};
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Instant;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ActiveTab {
    Statistics,
    Time,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum QuickSetRange {
    All,
    Today,
    Yesterday,
    ThisWeek,
    ThisMonth,
    ThisYear,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GraphPeriod {
    PerDay,
    PerWeek,
    PerMonth,
    PerYear,
}

pub struct ActLogApp {
    #[allow(dead_code)]
    state: Arc<SharedTrackerState>,
    db_path: String,

    // Navigation & Tabs
    active_tab: ActiveTab,

    // Left sidebar filter controls (1:1 Procrastitracker)
    expand_level: u32,
    filter_minutes: i64,
    substring_filter: String,
    quick_set: QuickSetRange,
    start_date: NaiveDate,
    end_date: NaiveDate,

    // Tagging
    filter_on_tag: bool,
    selected_tag_id: Option<i64>,
    selected_node_name: Option<String>,

    // Time Graph controls
    graph_period: GraphPeriod,
    graph_normalized: bool,

    // Data cache
    activities: Vec<ActivitySession>,
    tags: Vec<Tag>,
    tree_nodes: Vec<ActivityNode>,
    last_refresh: Instant,
}

impl ActLogApp {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String) -> Self {
        let today = Local::now().date_naive();
        let mut app = Self {
            state,
            db_path,
            active_tab: ActiveTab::Statistics,
            expand_level: 2,
            filter_minutes: 0,
            substring_filter: String::new(),
            quick_set: QuickSetRange::Today,
            start_date: today,
            end_date: today,
            filter_on_tag: false,
            selected_tag_id: None,
            selected_node_name: None,
            graph_period: GraphPeriod::PerDay,
            graph_normalized: false,
            activities: Vec::new(),
            tags: Vec::new(),
            tree_nodes: Vec::new(),
            last_refresh: Instant::now(),
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
        }
        self.rebuild_tree();
        self.last_refresh = Instant::now();
    }

    fn calculate_timestamps(&self) -> (i64, i64) {
        match self.quick_set {
            QuickSetRange::All => (0, i64::MAX),
            _ => {
                let s = Local
                    .from_local_datetime(&self.start_date.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap()
                    .timestamp();
                let e = Local
                    .from_local_datetime(&self.end_date.and_hms_opt(23, 59, 59).unwrap())
                    .unwrap()
                    .timestamp();
                (s, e)
            }
        }
    }

    fn rebuild_tree(&mut self) {
        let min_seconds = self.filter_minutes * 60;
        let tag_filter = if self.filter_on_tag {
            self.selected_tag_id
        } else {
            None
        };

        self.tree_nodes = ActivityNode::build_tree(
            &self.activities,
            self.expand_level,
            min_seconds,
            &self.substring_filter,
            tag_filter,
        );
    }

    fn apply_quick_set(&mut self, qs: QuickSetRange) {
        self.quick_set = qs;
        let today = Local::now().date_naive();
        match qs {
            QuickSetRange::All => {
                self.start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
                self.end_date = today;
            }
            QuickSetRange::Today => {
                self.start_date = today;
                self.end_date = today;
            }
            QuickSetRange::Yesterday => {
                let y = today - Duration::days(1);
                self.start_date = y;
                self.end_date = y;
            }
            QuickSetRange::ThisWeek => {
                let weekday = today.weekday().num_days_from_monday();
                self.start_date = today - Duration::days(weekday as i64);
                self.end_date = today;
            }
            QuickSetRange::ThisMonth => {
                self.start_date = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                self.end_date = today;
            }
            QuickSetRange::ThisYear => {
                self.start_date = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
                self.end_date = today;
            }
        }
        self.reload_data();
    }
}

impl eframe::App for ActLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set classic Windows Light Theme (gray dialog background, white content frames)
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = Color32::from_rgb(240, 240, 240);
        visuals.window_fill = Color32::from_rgb(240, 240, 240);
        ctx.set_visuals(visuals);

        // Auto reload data every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.reload_data();
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));

        egui::CentralPanel::default().show(ctx, |ui| {
            // 1. Top Tab Strip (Statistics / Time)
            self.render_tab_strip(ui);
            ui.add_space(4.0);

            // 2. Main split view: Left control panel + Right tree / graph panel
            ui.horizontal(|ui| {
                // Left Panel (195px fixed width)
                ui.allocate_ui_with_layout(
                    Vec2::new(195.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_left_controls(ui);
                    },
                );

                ui.separator();

                // Right Panel (Tree View or Time Graph)
                ui.vertical(|ui| {
                    match self.active_tab {
                        ActiveTab::Statistics => self.render_statistics_view(ui),
                        ActiveTab::Time => self.render_time_view(ui),
                    }
                });
            });
        });
    }
}

impl ActLogApp {
    fn render_tab_strip(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let stats_btn = ui.selectable_label(
                self.active_tab == ActiveTab::Statistics,
                egui::RichText::new("Statistics").size(13.0).strong(),
            );
            if stats_btn.clicked() {
                self.active_tab = ActiveTab::Statistics;
            }

            let time_btn = ui.selectable_label(
                self.active_tab == ActiveTab::Time,
                egui::RichText::new("Time").size(13.0).strong(),
            );
            if time_btn.clicked() {
                self.active_tab = ActiveTab::Time;
            }
        });
        ui.separator();
    }

    fn render_left_controls(&mut self, ui: &mut Ui) {
        let mut filter_changed = false;

        // Expanded Level Slider
        ui.label(egui::RichText::new("Expanded Level:").size(11.0));
        let slider = egui::Slider::new(&mut self.expand_level, 1..=5);
        if ui.add(slider).changed() {
            filter_changed = true;
        }
        ui.add_space(4.0);

        // Filter < Minutes
        ui.label(egui::RichText::new("Filter < Minutes:").size(11.0));
        let num_edit = egui::DragValue::new(&mut self.filter_minutes).range(0..=120);
        if ui.add(num_edit).changed() {
            filter_changed = true;
        }
        ui.add_space(4.0);

        // Filter on sub-string
        ui.label(egui::RichText::new("Filter on sub-string:").size(11.0));
        let text_box = egui::TextEdit::singleline(&mut self.substring_filter).desired_width(180.0);
        if ui.add(text_box).changed() {
            filter_changed = true;
        }
        ui.add_space(8.0);

        // Date Range GroupBox
        render_groupbox(ui, "Date Range", |ui| {
            ui.label(egui::RichText::new(format!("From: {}", self.start_date.format("%Y-%m-%d"))).size(11.0));
            ui.label(egui::RichText::new(format!("To:   {}", self.end_date.format("%Y-%m-%d"))).size(11.0));
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Quick Set:").size(11.0));
            let prev_qs = self.quick_set;
            egui::ComboBox::from_id_salt("quick_set_combo")
                .selected_text(match self.quick_set {
                    QuickSetRange::All => "All",
                    QuickSetRange::Today => "Today",
                    QuickSetRange::Yesterday => "Yesterday",
                    QuickSetRange::ThisWeek => "This Week",
                    QuickSetRange::ThisMonth => "This Month",
                    QuickSetRange::ThisYear => "This Year",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::All, "All");
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::Today, "Today");
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::Yesterday, "Yesterday");
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::ThisWeek, "This Week");
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::ThisMonth, "This Month");
                    ui.selectable_value(&mut self.quick_set, QuickSetRange::ThisYear, "This Year");
                });

            if self.quick_set != prev_qs {
                self.apply_quick_set(self.quick_set);
            }
        });

        ui.add_space(8.0);

        // Tagging GroupBox
        render_groupbox(ui, "Tagging", |ui| {
            if ui.checkbox(&mut self.filter_on_tag, "Filter on tag").changed() {
                filter_changed = true;
            }

            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .id_salt("tag_list_scroll")
                .max_height(160.0)
                .show(ui, |ui| {
                    for tag in &self.tags {
                        let is_sel = self.selected_tag_id == tag.id;
                        let tag_color = parse_hex_color(&tag.color);

                        ui.horizontal(|ui| {
                            let (swatch, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                            ui.painter().rect_filled(swatch, Rounding::ZERO, tag_color);
                            ui.painter().rect_stroke(swatch, Rounding::ZERO, Stroke::new(1.0, Color32::from_rgb(120, 120, 120)));

                            if ui.selectable_label(is_sel, egui::RichText::new(&tag.name).size(11.0)).clicked() {
                                self.selected_tag_id = tag.id;
                                if self.filter_on_tag {
                                    filter_changed = true;
                                }
                            }
                        });
                    }
                });

            ui.add_space(4.0);
            if ui.button("Apply Tag").clicked() {
                // Apply tag to current selected node
                if let (Some(ref node_name), Some(tag_id)) = (&self.selected_node_name, self.selected_tag_id) {
                    if let Ok(conn) = Connection::open(&self.db_path) {
                        let _ = conn.execute(
                            "UPDATE activities SET tag_id = ?1 WHERE app_name = ?2 OR window_title = ?2",
                            (tag_id, node_name),
                        );
                    }
                    self.reload_data();
                }
            }
        });

        if filter_changed {
            self.rebuild_tree();
        }
    }

    fn render_statistics_view(&mut self, ui: &mut Ui) {
        let total_active_sec: i64 = self
            .activities
            .iter()
            .filter(|a| !a.is_afk)
            .map(|a| a.duration)
            .sum();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Total Tracked Time: {} ({} activities)",
                    format_pt_time(total_active_sec),
                    self.activities.len()
                ))
                .strong()
                .size(12.0)
                .color(Color32::from_rgb(40, 40, 40)),
            );
        });

        ui.separator();

        // White framed tree area matching Procrastitracker
        let frame = egui::Frame::group(ui.style())
            .fill(Color32::WHITE)
            .rounding(Rounding::ZERO)
            .stroke(Stroke::new(1.0, Color32::from_rgb(127, 157, 185)))
            .inner_margin(egui::Margin::same(6.0));

        frame.show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("procrastitracker_tree_scroll")
                .show(ui, |ui| {
                    if self.tree_nodes.is_empty() {
                        ui.label(
                            egui::RichText::new("No activity records match the current filters.")
                                .color(Color32::GRAY)
                                .size(12.0),
                        );
                    } else {
                        render_procrastitracker_tree(
                            ui,
                            &mut self.tree_nodes,
                            &self.tags,
                            total_active_sec,
                            &mut self.selected_node_name,
                        );
                    }
                });
        });
    }

    fn render_time_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Period:").size(12.0));
            egui::ComboBox::from_id_salt("graph_period_combo")
                .selected_text(match self.graph_period {
                    GraphPeriod::PerDay => "Per Day",
                    GraphPeriod::PerWeek => "Per Week",
                    GraphPeriod::PerMonth => "Per Month",
                    GraphPeriod::PerYear => "Per Year",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerDay, "Per Day");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerWeek, "Per Week");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerMonth, "Per Month");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerYear, "Per Year");
                });

            ui.add_space(16.0);
            ui.checkbox(&mut self.graph_normalized, "Normalized");
        });

        ui.separator();

        let frame = egui::Frame::group(ui.style())
            .fill(Color32::WHITE)
            .rounding(Rounding::ZERO)
            .stroke(Stroke::new(1.0, Color32::from_rgb(127, 157, 185)))
            .inner_margin(egui::Margin::same(8.0));

        frame.show(ui, |ui| {
            ui.label(
                egui::RichText::new("Time Graph (Daily/Weekly Usage Stack)")
                    .strong()
                    .size(13.0),
            );
            ui.add_space(8.0);

            // Group by day for graph
            use std::collections::BTreeMap;
            let mut day_map: BTreeMap<String, i64> = BTreeMap::new();
            for act in &self.activities {
                if act.is_afk {
                    continue;
                }
                let day_str = chrono::DateTime::from_timestamp(act.start_time, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                *day_map.entry(day_str).or_insert(0) += act.duration;
            }

            let max_val = day_map.values().cloned().max().unwrap_or(1).max(1) as f32;

            ui.horizontal(|ui| {
                for (day_str, secs) in &day_map {
                    ui.vertical(|ui| {
                        let bar_height = if self.graph_normalized {
                            120.0
                        } else {
                            (120.0 * (*secs as f32 / max_val)).max(4.0)
                        };

                        let (rect, _) = ui.allocate_exact_size(Vec2::new(32.0, 120.0), egui::Sense::hover());
                        let fill_rect = egui::Rect::from_min_max(
                            egui::Pos2::new(rect.left(), rect.bottom() - bar_height),
                            rect.max,
                        );

                        ui.painter().rect_filled(rect, Rounding::ZERO, Color32::from_rgb(240, 240, 240));
                        ui.painter().rect_filled(fill_rect, Rounding::ZERO, Color32::from_rgb(96, 255, 96));
                        ui.painter().rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::from_rgb(180, 180, 180)));

                        ui.label(egui::RichText::new(&day_str[5..]).size(9.0));
                        ui.label(egui::RichText::new(format_pt_time(*secs)).size(9.0));
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }
}

fn render_groupbox(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(title)
                .strong()
                .size(11.0)
                .color(Color32::from_rgb(70, 70, 70)),
        );
        ui.add_space(2.0);
        add_contents(ui);
    });
}
