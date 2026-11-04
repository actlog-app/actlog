use crate::db::models::{ActivitySession, Tag};
use crate::db::repository::Repository;
use crate::tracker::engine::SharedTrackerState;
use crate::ui::theme::{configure_impeccable_theme, Theme};
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
    theme_configured: bool,
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
        if !self.theme_configured {
            configure_impeccable_theme(ctx);
            self.theme_configured = true;
        }

        // Auto reload data every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.reload_data();
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));

        egui::CentralPanel::default().show(ctx, |ui| {
            // 1. Top Bar: Segmented Tabs + Window Title
            self.render_top_header(ui);
            ui.add_space(8.0);

            // 2. Main split view: Left control panel + Right tree / graph panel
            ui.horizontal(|ui| {
                // Left Panel (210px width)
                ui.allocate_ui_with_layout(
                    Vec2::new(210.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_left_controls(ui);
                    },
                );

                ui.add_space(6.0);

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
    fn render_top_header(&mut self, ui: &mut Ui) {
        let theme = Theme::DARK;

        ui.horizontal(|ui| {
            // Segmented Pill Tabs
            let (rect, _) = ui.allocate_exact_size(Vec2::new(210.0, 32.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, Rounding::same(6.0), theme.surface);
            ui.painter().rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0, theme.border));

            let half_w = (rect.width() - 4.0) / 2.0;
            let stats_rect = egui::Rect::from_min_size(
                egui::Pos2::new(rect.left() + 2.0, rect.top() + 2.0),
                Vec2::new(half_w, rect.height() - 4.0),
            );
            let time_rect = egui::Rect::from_min_size(
                egui::Pos2::new(rect.left() + 2.0 + half_w, rect.top() + 2.0),
                Vec2::new(half_w, rect.height() - 4.0),
            );

            // Statistics Pill
            let stats_hovered = ui.rect_contains_pointer(stats_rect);
            if self.active_tab == ActiveTab::Statistics {
                ui.painter().rect_filled(stats_rect, Rounding::same(5.0), theme.accent);
            } else if stats_hovered {
                ui.painter().rect_filled(stats_rect, Rounding::same(5.0), theme.surface_hover);
            }
            if ui.put(stats_rect, egui::Button::new("Statistics").frame(false)).clicked() {
                self.active_tab = ActiveTab::Statistics;
            }

            // Time Pill
            let time_hovered = ui.rect_contains_pointer(time_rect);
            if self.active_tab == ActiveTab::Time {
                ui.painter().rect_filled(time_rect, Rounding::same(5.0), theme.accent);
            } else if time_hovered {
                ui.painter().rect_filled(time_rect, Rounding::same(5.0), theme.surface_hover);
            }
            if ui.put(time_rect, egui::Button::new("Time").frame(false)).clicked() {
                self.active_tab = ActiveTab::Time;
            }

            ui.add_space(12.0);

            // Subtitle
            ui.label(
                egui::RichText::new("ProcrastiTracker Statistics")
                    .size(13.0)
                    .color(theme.text_dim),
            );
        });
    }

    fn render_left_controls(&mut self, ui: &mut Ui) {
        let theme = Theme::DARK;
        let mut filter_changed = false;

        // Panel 1: Filter sliders & inputs
        render_card(ui, "FILTERING", |ui| {
            ui.label(egui::RichText::new("Expanded Level").size(11.0).color(theme.text_muted));
            let slider = egui::Slider::new(&mut self.expand_level, 1..=5).show_value(true);
            if ui.add(slider).changed() {
                filter_changed = true;
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Filter < Minutes").size(11.0).color(theme.text_muted));
            let num_edit = egui::DragValue::new(&mut self.filter_minutes)
                .range(0..=180)
                .speed(1.0);
            if ui.add(num_edit).changed() {
                filter_changed = true;
            }

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Filter on sub-string").size(11.0).color(theme.text_muted));
            let text_box = egui::TextEdit::singleline(&mut self.substring_filter)
                .hint_text("Search window or app...")
                .desired_width(190.0);
            if ui.add(text_box).changed() {
                filter_changed = true;
            }
        });

        ui.add_space(6.0);

        // Panel 2: Date Range
        render_card(ui, "DATE RANGE", |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("From:").size(11.0).color(theme.text_dim));
                ui.label(egui::RichText::new(self.start_date.format("%Y-%m-%d").to_string()).size(11.0).strong());
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("To:  ").size(11.0).color(theme.text_dim));
                ui.label(egui::RichText::new(self.end_date.format("%Y-%m-%d").to_string()).size(11.0).strong());
            });

            ui.add_space(4.0);
            ui.label(egui::RichText::new("Quick Set").size(11.0).color(theme.text_muted));

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
                .width(180.0)
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

        ui.add_space(6.0);

        // Panel 3: Tagging
        render_card(ui, "TAGGING", |ui| {
            if ui.checkbox(&mut self.filter_on_tag, "Filter on tag").changed() {
                filter_changed = true;
            }

            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .id_salt("tag_list_scroll")
                .max_height(170.0)
                .show(ui, |ui| {
                    for tag in &self.tags {
                        let is_sel = self.selected_tag_id == tag.id;
                        let tag_color = parse_hex_color(&tag.color);

                        ui.horizontal(|ui| {
                            let (swatch, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                            ui.painter().rect_filled(swatch, Rounding::same(3.0), tag_color);

                            if ui.selectable_label(is_sel, egui::RichText::new(&tag.name).size(11.5)).clicked() {
                                self.selected_tag_id = tag.id;
                                if self.filter_on_tag {
                                    filter_changed = true;
                                }
                            }
                        });
                    }
                });

            ui.add_space(6.0);
            if ui.button(egui::RichText::new("Apply Tag").strong()).clicked() {
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
        let theme = Theme::DARK;
        let total_active_sec: i64 = self
            .activities
            .iter()
            .filter(|a| !a.is_afk)
            .map(|a| a.duration)
            .sum();

        // Top summary bar
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Total Tracked Time: {}", format_pt_time(total_active_sec)))
                    .size(13.0)
                    .strong()
                    .color(theme.text_primary),
            );

            ui.label(
                egui::RichText::new(format!("•  {} items in view", self.activities.len()))
                    .size(12.0)
                    .color(theme.text_dim),
            );
        });

        ui.add_space(6.0);

        // Framed main tree area
        let frame = egui::Frame::group(ui.style())
            .fill(theme.surface)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::same(10.0));

        frame.show(ui, |ui| {
            egui::ScrollArea::both()
                .id_salt("procrastitracker_tree_scroll")
                .show(ui, |ui| {
                    if self.tree_nodes.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("No activity records match the current filters.")
                                    .color(theme.text_dim)
                                    .size(13.0),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("Adjust 'Filter < Minutes', search filter, or date range.")
                                    .color(theme.text_dim)
                                    .size(11.0),
                            );
                        });
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
        let theme = Theme::DARK;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Period:").size(12.0).color(theme.text_muted));
            egui::ComboBox::from_id_salt("graph_period_combo")
                .selected_text(match self.graph_period {
                    GraphPeriod::PerDay => "Per Day",
                    GraphPeriod::PerWeek => "Per Week",
                    GraphPeriod::PerMonth => "Per Month",
                    GraphPeriod::PerYear => "Per Year",
                })
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerDay, "Per Day");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerWeek, "Per Week");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerMonth, "Per Month");
                    ui.selectable_value(&mut self.graph_period, GraphPeriod::PerYear, "Per Year");
                });

            ui.add_space(16.0);
            ui.checkbox(&mut self.graph_normalized, "Normalized");
        });

        ui.add_space(6.0);

        let frame = egui::Frame::group(ui.style())
            .fill(theme.surface)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(egui::Margin::same(12.0));

        frame.show(ui, |ui| {
            ui.label(
                egui::RichText::new("Time Graph (Daily/Weekly Usage Breakdown)")
                    .strong()
                    .size(13.0)
                    .color(theme.text_primary),
            );
            ui.add_space(12.0);

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
                            140.0
                        } else {
                            (140.0 * (*secs as f32 / max_val)).max(4.0)
                        };

                        let (rect, _) = ui.allocate_exact_size(Vec2::new(36.0, 140.0), egui::Sense::hover());
                        let fill_rect = egui::Rect::from_min_max(
                            egui::Pos2::new(rect.left(), rect.bottom() - bar_height),
                            rect.max,
                        );

                        ui.painter().rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(28, 32, 40));
                        ui.painter().rect_filled(fill_rect, Rounding::same(4.0), Color32::from_rgb(96, 255, 96));
                        ui.painter().rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0, theme.border));

                        ui.label(egui::RichText::new(&day_str[5..]).size(10.0).color(theme.text_dim));
                        ui.label(egui::RichText::new(format_pt_time(*secs)).size(10.0).monospace().color(theme.text_muted));
                    });
                    ui.add_space(6.0);
                }
            });
        });
    }
}

fn render_card(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    let theme = Theme::DARK;
    let frame = egui::Frame::group(ui.style())
        .fill(theme.surface)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0));

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .strong()
                .color(theme.text_dim),
        );
        ui.add_space(4.0);
        add_contents(ui);
    });
}
