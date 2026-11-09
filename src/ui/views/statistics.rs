use crate::model::{ActivitySession, Tag};
use crate::ui::common::{
    draw_dotted_h_line, draw_dotted_v_line, format_pt_time, render_win32_groupbox,
    resolve_tag_color,
};
use crate::ui::theme::Theme;
use crate::ui::views::day_graph::render_day_minimap;
use chrono::{Datelike, Duration, Local, NaiveDate};
use egui::{Color32, FontId, Margin, Pos2, Rect, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSet {
    Today,
    Yesterday,
    SinceMondayMorning,
    Last7Days,
    Last14Days,
    MonthToDate,
    ThirtyDays,
    NinetyDays,
    YearToDate,
    SinceAYearAgo,
    AllTime,
}

impl QuickSet {
    pub const ALL: &'static [QuickSet] = &[
        QuickSet::Today,
        QuickSet::Yesterday,
        QuickSet::SinceMondayMorning,
        QuickSet::Last7Days,
        QuickSet::Last14Days,
        QuickSet::MonthToDate,
        QuickSet::ThirtyDays,
        QuickSet::NinetyDays,
        QuickSet::YearToDate,
        QuickSet::SinceAYearAgo,
        QuickSet::AllTime,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            QuickSet::Today => "Today",
            QuickSet::Yesterday => "Yesterday",
            QuickSet::SinceMondayMorning => "Since Monday Morning",
            QuickSet::Last7Days => "Last 7 Days",
            QuickSet::Last14Days => "Last 14 Days",
            QuickSet::MonthToDate => "Month To Date",
            QuickSet::ThirtyDays => "30 Days",
            QuickSet::NinetyDays => "90 Days",
            QuickSet::YearToDate => "Year To Date",
            QuickSet::SinceAYearAgo => "Since A Year Ago",
            QuickSet::AllTime => "All Time",
        }
    }

    pub fn compute_range(&self, today: NaiveDate) -> (NaiveDate, NaiveDate) {
        match self {
            QuickSet::Today => (today, today),
            QuickSet::Yesterday => (today - Duration::days(1), today - Duration::days(1)),
            QuickSet::SinceMondayMorning => {
                let weekday = today.weekday().num_days_from_monday();
                let days_back = if weekday == 0 { 7 } else { weekday };
                (today - Duration::days(days_back as i64), today)
            }
            QuickSet::Last7Days => (today - Duration::days(6), today),
            QuickSet::Last14Days => (today - Duration::days(13), today),
            QuickSet::MonthToDate => {
                let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
                (start, today)
            }
            QuickSet::ThirtyDays => (today - Duration::days(29), today),
            QuickSet::NinetyDays => (today - Duration::days(89), today),
            QuickSet::YearToDate => {
                let start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap_or(today);
                (start, today)
            }
            QuickSet::SinceAYearAgo => (today - Duration::days(365), today),
            QuickSet::AllTime => (NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(), today),
        }
    }
}

/// Recursive tree node for hierarchical process/window/url display.
#[derive(Debug, Clone)]
pub struct ActivityNode {
    pub name: String,
    pub total_seconds: i64,
    pub tag_id: Option<i64>,
    pub tag_breakdown: BTreeMap<Option<i64>, i64>,
    pub children: Vec<ActivityNode>,
    pub is_expanded: bool,
}

const SEPARATORS: &[&str] = &[" - ", " | ", " : ", " > ", "\\\\", "\\", "//", "/"];

/// Splits a raw activity title into tokens matching PT nodedb.h:addtodatabase
fn tokenize_activity(act: &ActivitySession) -> Vec<String> {
    let mut clean_app = act.app_name.to_lowercase();
    if clean_app.ends_with(".exe") {
        clean_app.truncate(clean_app.len() - 4);
    }

    let mut raw_full = clean_app;
    if let Some(ref url) = act.url {
        if !url.is_empty() {
            raw_full.push_str(" - ");
            raw_full.push_str(url);
        }
    } else if let Some(ref domain) = act.domain {
        if !domain.is_empty() {
            raw_full.push_str(" - ");
            raw_full.push_str(domain);
        }
    }

    if !act.window_title.is_empty() {
        raw_full.push_str(" - ");
        raw_full.push_str(&act.window_title);
    }

    let mut tokens = Vec::new();
    let mut remaining = raw_full.as_str();

    while !remaining.is_empty() {
        let mut earliest_pos = None;
        let mut chosen_sep_len = 0;

        for &sep in SEPARATORS {
            if let Some(pos) = remaining.find(sep) {
                if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                    earliest_pos = Some(pos);
                    chosen_sep_len = sep.len();
                }
            }
        }

        match earliest_pos {
            Some(pos) => {
                let token = remaining[..pos].trim();
                if !token.is_empty() && token != "[" {
                    let clean = token.strip_prefix('[').unwrap_or(token).trim();
                    if !clean.is_empty() {
                        tokens.push(clean.to_string());
                    }
                }
                remaining = &remaining[pos + chosen_sep_len..];
            }
            None => {
                let token = remaining.trim();
                if !token.is_empty() && token != "[" {
                    let clean = token.strip_prefix('[').unwrap_or(token).trim();
                    if !clean.is_empty() {
                        tokens.push(clean.to_string());
                    }
                }
                break;
            }
        }
    }

    if tokens.is_empty() {
        tokens.push(act.app_name.to_lowercase());
    }
    tokens
}

#[derive(Default)]
struct TempNode {
    total_seconds: i64,
    tag_id: Option<i64>,
    tag_breakdown: BTreeMap<Option<i64>, i64>,
    children: BTreeMap<String, TempNode>,
}

impl TempNode {
    fn insert(&mut self, tokens: &[String], duration: i64, tag_id: Option<i64>) {
        self.total_seconds += duration;
        *self.tag_breakdown.entry(tag_id).or_insert(0) += duration;
        if self.tag_id.is_none() {
            self.tag_id = tag_id;
        }

        if let Some((first, rest)) = tokens.split_first() {
            let child = self.children.entry(first.clone()).or_default();
            child.insert(rest, duration, tag_id);
        }
    }

    fn to_activity_node(
        name: String,
        temp: TempNode,
        expand_level: u32,
        current_depth: u32,
        min_seconds: i64,
        substring_filter: &str,
        expanded_overrides: &std::collections::HashMap<String, bool>,
    ) -> Option<ActivityNode> {
        if temp.total_seconds < min_seconds {
            return None;
        }

        let mut children = Vec::new();
        for (child_name, child_temp) in temp.children {
            if let Some(c) = Self::to_activity_node(
                child_name,
                child_temp,
                expand_level,
                current_depth + 1,
                min_seconds,
                substring_filter,
                expanded_overrides,
            ) {
                children.push(c);
            }
        }

        let filter_lower = substring_filter.to_lowercase();
        if !filter_lower.is_empty() {
            let matches_self = name.to_lowercase().contains(&filter_lower);
            let has_matching_child = !children.is_empty();
            if !matches_self && !has_matching_child {
                return None;
            }
        }

        children.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        // 1:1 ProcrastiTracker node.h:372 single-child collapse
        let mut curr_name = name;
        let mut curr_children = children;
        let mut curr_tag_id = temp.tag_id;
        let mut curr_tag_breakdown = temp.tag_breakdown;

        while curr_children.len() == 1 {
            let child = curr_children.remove(0);
            curr_name = format!("{} - {}", curr_name, child.name);
            curr_tag_id = child.tag_id.or(curr_tag_id);
            curr_tag_breakdown = child.tag_breakdown;
            curr_children = child.children;
        }

        let default_expanded = current_depth < expand_level;
        let is_expanded = *expanded_overrides.get(&curr_name).unwrap_or(&default_expanded);

        Some(ActivityNode {
            name: curr_name,
            total_seconds: temp.total_seconds,
            tag_id: curr_tag_id,
            tag_breakdown: curr_tag_breakdown,
            children: curr_children,
            is_expanded,
        })
    }
}

impl ActivityNode {
    pub fn build_tree(
        activities: &[ActivitySession],
        expand_level: u32,
        min_seconds: i64,
        substring_filter: &str,
        tag_filter: Option<i64>,
        expanded_overrides: &std::collections::HashMap<String, bool>,
    ) -> Vec<ActivityNode> {
        let mut root = TempNode::default();

        for act in activities {
            if act.is_afk || act.duration <= 0 {
                continue;
            }
            if let Some(target_tag) = tag_filter {
                if act.tag_id != Some(target_tag) {
                    continue;
                }
            }

            let tokens = tokenize_activity(act);
            root.insert(&tokens, act.duration, act.tag_id);
        }

        let mut roots = Vec::new();
        for (name, temp) in root.children {
            if let Some(node) = TempNode::to_activity_node(
                name,
                temp,
                expand_level,
                1,
                min_seconds,
                substring_filter,
                expanded_overrides,
            ) {
                roots.push(node);
            }
        }

        roots.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

        if roots.is_empty() {
            return Vec::new();
        }

        // Canonical ProcrastiTracker Root Node: "[-] <total_time> - (root)"
        let total_seconds: i64 = roots.iter().map(|n| n.total_seconds).sum();
        let mut total_breakdown = BTreeMap::new();
        for n in &roots {
            for (tid, sec) in &n.tag_breakdown {
                *total_breakdown.entry(*tid).or_insert(0) += *sec;
            }
        }

        let root_expanded = *expanded_overrides.get("(root)").unwrap_or(&true);

        vec![ActivityNode {
            name: "(root)".to_string(),
            total_seconds,
            tag_id: None,
            tag_breakdown: total_breakdown,
            children: roots,
            is_expanded: root_expanded,
        }]
    }
}

pub struct StatisticsControlsState {
    pub expand_level: u32,
    pub filter_min_minutes: i64,
    pub filter_substring: String,
    pub quick_set: QuickSet,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_date_str: String,
    pub end_date_str: String,
    pub filter_on_tag: bool,
    pub selected_tag_id: Option<i64>,
    pub selected_node_name: Option<String>,
    pub renaming_tag_id: Option<i64>,
    pub rename_buffer: String,
    pub hover_status: Option<String>,
    pub expanded_overrides: std::collections::HashMap<String, bool>,
    pub show_start_calendar: bool,
    pub show_end_calendar: bool,

    // Performance Caching (eliminates per-frame rebuilding & lag)
    pub cached_tree: Vec<ActivityNode>,
    pub cached_max_seconds: f32,
    pub cached_day_stats: Vec<crate::ui::views::day_graph::DayStat>,
    pub is_dirty: bool,
}

impl StatisticsControlsState {
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn sync_cache(&mut self, activities: &[ActivitySession]) {
        if self.is_dirty || self.cached_tree.is_empty() {
            let tag_filter = if self.filter_on_tag {
                self.selected_tag_id
            } else {
                None
            };
            let min_sec = self.filter_min_minutes * 60;

            self.cached_tree = ActivityNode::build_tree(
                activities,
                self.expand_level,
                min_sec,
                &self.filter_substring,
                tag_filter,
                &self.expanded_overrides,
            );

            // Bargraph maximum is top-level child (app with most time) matching PT statview.h:399
            self.cached_max_seconds = if !self.cached_tree.is_empty() && !self.cached_tree[0].children.is_empty() {
                self.cached_tree[0].children[0].total_seconds.max(1) as f32
            } else if !self.cached_tree.is_empty() {
                self.cached_tree[0].total_seconds.max(1) as f32
            } else {
                1.0
            };

            self.cached_day_stats = crate::ui::views::day_graph::DayStat::compute_active_days(
                activities,
                self.start_date,
                self.end_date,
            );

            self.is_dirty = false;
        }
    }

    pub fn sync_date_strings(&mut self) {
        self.start_date_str = format!(
            "{}/{}/{}",
            self.start_date.month(),
            self.start_date.day(),
            self.start_date.year()
        );
        self.end_date_str = format!(
            "{}/{}/{}",
            self.end_date.month(),
            self.end_date.day(),
            self.end_date.year()
        );
        self.is_dirty = true;
    }
}

impl Default for StatisticsControlsState {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let mut s = Self {
            expand_level: 1,
            filter_min_minutes: 0,
            filter_substring: String::new(),
            quick_set: QuickSet::Today,
            start_date: today,
            end_date: today,
            start_date_str: String::new(),
            end_date_str: String::new(),
            filter_on_tag: false,
            selected_tag_id: None,
            selected_node_name: Some("(root)".to_string()),
            renaming_tag_id: None,
            rename_buffer: String::new(),
            hover_status: None,
            expanded_overrides: std::collections::HashMap::new(),
            show_start_calendar: false,
            show_end_calendar: false,
            cached_tree: Vec::new(),
            cached_max_seconds: 1.0,
            cached_day_stats: Vec::new(),
            is_dirty: true,
        };
        s.sync_date_strings();
        s
    }
}

fn parse_user_date(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%-m/%-d/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(d);
    }
    None
}

pub fn render_statistics_view(
    ui: &mut Ui,
    state: &mut StatisticsControlsState,
    activities: &[ActivitySession],
    tags: &[Tag],
    date_changed: &mut bool,
    tag_assigned: &mut Option<(String, Option<i64>)>,
    tag_renamed: &mut Option<(i64, String)>,
) {
    let theme = Theme::WIN32;
    let today = Local::now().date_naive();

    ui.horizontal(|ui| {
        // --- 1. LEFT SIDEBAR (~185px) ---
        ui.vertical(|ui| {
            ui.set_width(185.0);

            // 1. Expanded Level Slider
            ui.label(RichText::new("Expanded Level:").size(12.0).color(theme.text_primary));
            let old_level = state.expand_level;
            ui.add(egui::Slider::new(&mut state.expand_level, 1..=5).show_value(false));
            if state.expand_level != old_level {
                state.expanded_overrides.clear();
                state.is_dirty = true;
            }

            ui.add_space(4.0);

            // 2. Filter < Minutes:
            ui.label(RichText::new("Filter < Minutes:").size(12.0).color(theme.text_primary));
            let mut min_str = state.filter_min_minutes.to_string();
            let min_resp = ui.add(
                egui::TextEdit::singleline(&mut min_str)
                    .desired_width(175.0)
                    .margin(Margin::symmetric(4.0, 2.0)),
            );
            if min_resp.changed() {
                if let Ok(v) = min_str.trim().parse::<i64>() {
                    state.filter_min_minutes = v.max(0);
                    state.is_dirty = true;
                }
            }

            ui.add_space(4.0);

            // 3. Filter on sub-strings (1:1 with photo1 without colon)
            ui.label(RichText::new("Filter on sub-strings").size(12.0).color(theme.text_primary));
            let sub_resp = ui.add(
                egui::TextEdit::singleline(&mut state.filter_substring)
                    .desired_width(175.0)
                    .margin(Margin::symmetric(4.0, 2.0)),
            );
            if sub_resp.changed() {
                state.is_dirty = true;
            }

            ui.add_space(4.0);

            // 4. Date Range GroupBox (1:1 with photo1 Win32 DateTimePickers)
            render_win32_groupbox(ui, "Date Range", |ui| {
                // Start Date Input Row
                ui.horizontal(|ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.start_date_str)
                            .desired_width(138.0)
                            .margin(Margin::symmetric(3.0, 2.0)),
                    );
                    if resp.lost_focus() {
                        if let Some(parsed) = parse_user_date(&state.start_date_str) {
                            state.start_date = parsed;
                            state.sync_date_strings();
                            *date_changed = true;
                        } else {
                            state.sync_date_strings();
                        }
                    }

                    // Win32 [📅] Drop-down / Calendar Picker button
                    let cal_btn = ui.add(
                        egui::Button::new(RichText::new("📅").size(11.0))
                            .min_size(Vec2::new(22.0, 19.0)),
                    );
                    if cal_btn.clicked() {
                        state.show_start_calendar = !state.show_start_calendar;
                        state.show_end_calendar = false;
                    }
                });

                if state.show_start_calendar {
                    render_calendar_picker(ui, &mut state.start_date, &mut state.show_start_calendar, date_changed);
                    if *date_changed {
                        state.sync_date_strings();
                    }
                }

                ui.add_space(2.0);

                // End Date Input Row
                ui.horizontal(|ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.end_date_str)
                            .desired_width(138.0)
                            .margin(Margin::symmetric(3.0, 2.0)),
                    );
                    if resp.lost_focus() {
                        if let Some(parsed) = parse_user_date(&state.end_date_str) {
                            state.end_date = parsed;
                            state.sync_date_strings();
                            *date_changed = true;
                        } else {
                            state.sync_date_strings();
                        }
                    }

                    let cal_btn = ui.add(
                        egui::Button::new(RichText::new("📅").size(11.0))
                            .min_size(Vec2::new(22.0, 19.0)),
                    );
                    if cal_btn.clicked() {
                        state.show_end_calendar = !state.show_end_calendar;
                        state.show_start_calendar = false;
                    }
                });

                if state.show_end_calendar {
                    render_calendar_picker(ui, &mut state.end_date, &mut state.show_end_calendar, date_changed);
                    if *date_changed {
                        state.sync_date_strings();
                    }
                }

                ui.add_space(2.0);
                ui.label(RichText::new("Quick Set:").size(12.0).color(theme.text_primary));

                egui::ComboBox::from_id_salt("win32_quick_set_combo")
                    .selected_text(state.quick_set.label())
                    .width(166.0)
                    .show_ui(ui, |ui| {
                        for qs in QuickSet::ALL {
                            if ui.selectable_value(&mut state.quick_set, *qs, qs.label()).clicked() {
                                let (start, end) = state.quick_set.compute_range(today);
                                state.start_date = start;
                                state.end_date = end;
                                state.sync_date_strings();
                                *date_changed = true;
                            }
                        }
                    });
            });

            ui.add_space(4.0);

            // 5. Tagging GroupBox (1:1 with photo1 Win32 IDC_LIST1)
            render_win32_groupbox(ui, "Tagging", |ui| {
                if ui.checkbox(&mut state.filter_on_tag, "Filter on tag").changed() {
                    state.is_dirty = true;
                }
                ui.add_space(2.0);

                let has_node_sel =
                    state.selected_node_name.is_some() && state.selected_tag_id.is_some();
                let apply_btn = ui.add_enabled(
                    has_node_sel,
                    egui::Button::new(RichText::new("Apply Tag to Node").size(12.0))
                        .min_size(Vec2::new(166.0, 19.0)),
                );
                if apply_btn.clicked() {
                    if let (Some(node_name), Some(tag_id)) =
                        (&state.selected_node_name, state.selected_tag_id)
                    {
                        *tag_assigned = Some((node_name.clone(), Some(tag_id)));
                        state.is_dirty = true;
                    }
                }

                let has_tag_sel = state.selected_tag_id.is_some();
                let rename_btn = ui.add_enabled(
                    has_tag_sel,
                    egui::Button::new(RichText::new("Rename Tag").size(12.0))
                        .min_size(Vec2::new(166.0, 19.0)),
                );
                if rename_btn.clicked() {
                    if let Some(tag_id) = state.selected_tag_id {
                        state.renaming_tag_id = Some(tag_id);
                        if let Some(t) = tags.iter().find(|t| t.id == Some(tag_id)) {
                            state.rename_buffer = t.name.clone();
                        }
                    }
                }

                if let Some(tag_id) = state.renaming_tag_id {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.rename_buffer)
                                .desired_width(115.0),
                        );
                        if ui.button("OK").clicked() {
                            let new_name = state.rename_buffer.trim().to_string();
                            if !new_name.is_empty() {
                                *tag_renamed = Some((tag_id, new_name));
                            }
                            state.renaming_tag_id = None;
                        }
                    });
                }

                ui.add_space(3.0);

                // 15 Canonical Tags List (SysListView32 with 9x9 square swatches, 11.5px row)
                egui::Frame::none()
                    .fill(theme.control_bg)
                    .stroke(Stroke::new(1.0, theme.control_border))
                    .inner_margin(Margin::same(2.0))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_salt("win32_tag_list_scroll")
                            .max_height(160.0)
                            .min_scrolled_height(155.0)
                            .show(ui, |ui| {
                                for tag in tags {
                                    let tag_id = tag.id.unwrap_or(0);
                                    let is_sel = state.selected_tag_id == Some(tag_id);
                                    let color = crate::ui::common::parse_hex_color(&tag.color);

                                    let (rect, row_resp) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 11.5),
                                        egui::Sense::click(),
                                    );

                                    if is_sel {
                                        ui.painter().rect_filled(
                                            rect,
                                            Rounding::ZERO,
                                            theme.selection_bg,
                                        );
                                    }

                                    // 9x9 Swatch
                                    let swatch_rect = Rect::from_min_size(
                                        Pos2::new((rect.left() + 2.0).round(), (rect.top() + 1.0).round()),
                                        Vec2::new(9.0, 9.0),
                                    );
                                    ui.painter().rect_filled(swatch_rect, Rounding::ZERO, color);
                                    ui.painter().rect_stroke(
                                        swatch_rect,
                                        Rounding::ZERO,
                                        Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
                                    );

                                    // Tag Name in crisp 12.0 font
                                    let text_color = if is_sel {
                                        theme.selection_fg
                                    } else {
                                        theme.text_primary
                                    };
                                    ui.painter().text(
                                        Pos2::new((rect.left() + 15.0).round(), (rect.top() - 1.0).round()),
                                        egui::Align2::LEFT_TOP,
                                        &tag.name,
                                        FontId::proportional(12.0),
                                        text_color,
                                    );

                                    if row_resp.clicked() {
                                        state.selected_tag_id =
                                            if is_sel { None } else { Some(tag_id) };
                                        if state.filter_on_tag {
                                            state.is_dirty = true;
                                        }
                                    }
                                }
                            });
                    });
            });

            ui.add_space(6.0);

            // 6. Dynamic Bottom Status Label ("Hover over graph..." or "YYYY-M-D -> HH:MM:SS")
            let status_text = state
                .hover_status
                .as_deref()
                .unwrap_or("Hover over graph...");
            ui.label(RichText::new(status_text).size(12.0).color(theme.text_primary));
        });

        ui.add_space(4.0);

        // Synchronize cached tree and day stats only when filters or activities changed!
        state.sync_cache(activities);

        let StatisticsControlsState {
            ref mut cached_tree,
            cached_max_seconds,
            ref mut selected_node_name,
            ref mut expanded_overrides,
            ref cached_day_stats,
            ref mut hover_status,
            ..
        } = state;

        let max_seconds = *cached_max_seconds;

        // --- 2. CENTER PANEL: TREEVIEW WITH FULL-HEIGHT IN-ROW BARS (Flex Width) ---
        let center_width = (ui.available_width() - 96.0).max(150.0);
        let avail_h = ui.available_height();
        ui.vertical(|ui| {
            ui.set_width(center_width);
            ui.set_min_height(avail_h);

            egui::Frame::none()
                .fill(theme.control_bg)
                .stroke(Stroke::new(1.0, theme.control_border))
                .inner_margin(Margin::same(4.0))
                .show(ui, |ui| {
                    ui.set_min_height(avail_h - 12.0);
                    if cached_tree.is_empty() {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(
                                    "No activities match the current filters or date range.",
                                )
                                .size(12.0)
                                .color(theme.text_dim),
                            );
                        });
                    } else {
                        ScrollArea::both()
                            .id_salt("win32_tree_scroll")
                            .show(ui, |ui| {
                                let mut ancestor_has_more = Vec::new();
                                render_tree_nodes(
                                    ui,
                                    cached_tree,
                                    tags,
                                    max_seconds,
                                    0,
                                    selected_node_name,
                                    expanded_overrides,
                                    &mut ancestor_has_more,
                                );
                            });
                    }
                });
        });

        ui.add_space(4.0);

        // --- 3. RIGHT PANEL: VERTICAL DAY MINIMAP GRAPH (~90px) ---
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(avail_h);
            render_day_minimap(
                ui,
                cached_day_stats,
                tags,
                hover_status,
            );
        });
    });
}

fn render_calendar_picker(
    ui: &mut Ui,
    date: &mut NaiveDate,
    show_popup: &mut bool,
    date_changed: &mut bool,
) {
    let theme = Theme::WIN32;
    egui::Frame::none()
        .fill(Color32::from_rgb(255, 255, 255))
        .stroke(Stroke::new(1.0, theme.control_border))
        .inner_margin(Margin::same(4.0))
        .show(ui, |ui| {
            ui.set_width(170.0);
            ui.horizontal(|ui| {
                if ui.small_button("◀").clicked() {
                    let prev_month = if date.month() == 1 { 12 } else { date.month() - 1 };
                    let prev_year = if date.month() == 1 { date.year() - 1 } else { date.year() };
                    if let Some(d) = NaiveDate::from_ymd_opt(prev_year, prev_month, 1) {
                        *date = d;
                        *date_changed = true;
                    }
                }
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new(format!("{:?} {}", date.month(), date.year()))
                            .size(11.0)
                            .strong(),
                    );
                });
                if ui.small_button("▶").clicked() {
                    let next_month = if date.month() == 12 { 1 } else { date.month() + 1 };
                    let next_year = if date.month() == 12 { date.year() + 1 } else { date.year() };
                    if let Some(d) = NaiveDate::from_ymd_opt(next_year, next_month, 1) {
                        *date = d;
                        *date_changed = true;
                    }
                }
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                for day_name in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                    ui.add_sized(
                        [20.0, 14.0],
                        egui::Label::new(RichText::new(day_name).size(9.0).color(theme.text_dim)),
                    );
                }
            });

            let year = date.year();
            let month = date.month();
            let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(*date);
            let start_weekday = first_day.weekday().num_days_from_sunday() as usize;

            let next_month_date = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
            };
            let days_in_month = (next_month_date - first_day).num_days() as u32;

            egui::Grid::new(ui.id().with("cal_grid"))
                .spacing([2.0, 2.0])
                .show(ui, |ui| {
                    let mut cell = 0;
                    for _ in 0..start_weekday {
                        ui.add_sized([20.0, 16.0], egui::Label::new(""));
                        cell += 1;
                    }
                    for day_num in 1..=days_in_month {
                        let d = NaiveDate::from_ymd_opt(year, month, day_num).unwrap();
                        let is_current = d == *date;
                        let btn = egui::Button::new(RichText::new(day_num.to_string()).size(10.0))
                            .min_size(Vec2::new(20.0, 16.0));
                        let resp = if is_current {
                            ui.add(btn.fill(Color32::from_rgb(205, 232, 255)))
                        } else {
                            ui.add(btn)
                        };
                        if resp.clicked() {
                            *date = d;
                            *date_changed = true;
                            *show_popup = false;
                        }
                        cell += 1;
                        if cell % 7 == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(2.0);
            if ui.button("Today").clicked() {
                *date = Local::now().date_naive();
                *date_changed = true;
                *show_popup = false;
            }
        });
}

pub fn render_tree_nodes(
    ui: &mut Ui,
    nodes: &mut [ActivityNode],
    tags: &[Tag],
    max_seconds: f32,
    depth: usize,
    selected_node_name: &mut Option<String>,
    expanded_overrides: &mut std::collections::HashMap<String, bool>,
    ancestor_has_more: &mut Vec<bool>,
) {
    let num_nodes = nodes.len();
    for (i, node) in nodes.iter_mut().enumerate() {
        let is_last_sibling = i + 1 == num_nodes;
        render_tree_node(
            ui,
            node,
            tags,
            max_seconds,
            depth,
            selected_node_name,
            expanded_overrides,
            ancestor_has_more,
            is_last_sibling,
        );
    }
}

fn render_tree_node(
    ui: &mut Ui,
    node: &mut ActivityNode,
    tags: &[Tag],
    max_seconds: f32,
    depth: usize,
    selected_node_name: &mut Option<String>,
    expanded_overrides: &mut std::collections::HashMap<String, bool>,
    ancestor_has_more: &mut Vec<bool>,
    is_last_sibling: bool,
) {
    let has_children = !node.children.is_empty();
    let _theme = Theme::WIN32;
    let row_height = 18.0;

    // Depth indentation (16px per level)
    let indent_x = (depth as f32) * 16.0;

    let (row_rect, row_resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), row_height),
        egui::Sense::click(),
    );

    let is_selected = selected_node_name.as_deref() == Some(&node.name);

    if row_resp.clicked() {
        *selected_node_name = Some(node.name.clone());
    }

    let painter = ui.painter();
    let row_mid_y = row_rect.center().y;

    // 1. Dotted Tree Guidelines (1:1 with TVS_HASLINES)
    if depth > 0 {
        let line_color = Color32::from_rgb(160, 160, 160);

        // Ongoing vertical lines for ancestors that have more siblings below
        for (lvl, &has_more) in ancestor_has_more.iter().enumerate() {
            if has_more && lvl + 1 < depth {
                let anc_x = row_rect.left() + (lvl as f32) * 16.0 + 5.0;
                draw_dotted_v_line(painter, anc_x, row_rect.top(), row_rect.bottom(), line_color);
            }
        }

        // Immediate parent vertical guideline
        let parent_lvl = depth - 1;
        let parent_x = row_rect.left() + (parent_lvl as f32) * 16.0 + 5.0;
        let v_end = if is_last_sibling {
            row_mid_y
        } else {
            row_rect.bottom()
        };
        draw_dotted_v_line(painter, parent_x, row_rect.top(), v_end, line_color);

        // Horizontal connector from parent guideline to this node
        let h_end = if has_children {
            row_rect.left() + indent_x + 0.5
        } else {
            row_rect.left() + indent_x + 13.0 // connects directly to leaf node text!
        };
        draw_dotted_h_line(painter, parent_x, h_end, row_mid_y, line_color);
    }

    // 2. Square [+] / [-] Toggle Button
    let toggle_center_x = row_rect.left() + indent_x + 5.0;
    if has_children {
        let toggle_rect = Rect::from_center_size(
            Pos2::new(toggle_center_x, row_mid_y),
            Vec2::new(9.0, 9.0),
        );
        let toggle_resp = ui.interact(toggle_rect, ui.id().with(&node.name), egui::Sense::click());

        // Draw 9x9 box
        painter.rect_filled(toggle_rect, Rounding::ZERO, Color32::from_rgb(255, 255, 255));
        painter.rect_stroke(
            toggle_rect,
            Rounding::ZERO,
            Stroke::new(1.0, Color32::from_rgb(128, 128, 128)),
        );

        // Minus line
        painter.line_segment(
            [
                Pos2::new(toggle_rect.left() + 2.0, row_mid_y),
                Pos2::new(toggle_rect.right() - 2.0, row_mid_y),
            ],
            Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
        );
        // Plus vertical line if collapsed
        if !node.is_expanded {
            painter.line_segment(
                [
                    Pos2::new(toggle_center_x, toggle_rect.top() + 2.0),
                    Pos2::new(toggle_center_x, toggle_rect.bottom() - 2.0),
                ],
                Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),
            );
        }

        if toggle_resp.clicked() {
            node.is_expanded = !node.is_expanded;
            expanded_overrides.insert(node.name.clone(), node.is_expanded);
        }
    }

    // 3. Selection Highlight (1:1 with Win32 ListView/TreeView selection)
    if is_selected {
        let sel_bg = Color32::from_rgb(205, 232, 255);
        let sel_stroke = Color32::from_rgb(153, 209, 255);
        painter.rect_filled(row_rect, Rounding::ZERO, sel_bg);
        painter.rect_stroke(
            row_rect.shrink(0.5),
            Rounding::ZERO,
            Stroke::new(1.0, sel_stroke),
        );
    }

    // 4. Text Position & In-Row Colored Bar (1:1 with PT handleCustomDraw BitBlt)
    let text_x = row_rect.left() + indent_x + 14.0;
    let formatted_time = format_pt_time(node.total_seconds);
    let display_text = format!("{} - {}", formatted_time, node.name);

    let is_root_or_expanded_parent = node.name == "(root)" || (has_children && node.is_expanded);

    if !is_root_or_expanded_parent {
        // Collapsed or leaf node: Draw Full-Height In-Row Colored Tag Bar directly behind text!
        let avail_w = (row_rect.right() - text_x - 4.0).max(20.0);
        let total_bar_w = ((node.total_seconds as f32 / max_seconds) * avail_w).clamp(2.0, avail_w);

        let mut curr_x = text_x;
        if node.tag_breakdown.is_empty() {
            let seg_rect = Rect::from_min_max(
                Pos2::new(curr_x, row_rect.top()),
                Pos2::new(curr_x + total_bar_w, row_rect.bottom()),
            );
            painter.rect_filled(seg_rect, Rounding::ZERO, resolve_tag_color(None, tags));
        } else {
            for (tid, sec) in &node.tag_breakdown {
                let seg_w =
                    ((*sec as f32 / node.total_seconds.max(1) as f32) * total_bar_w).max(1.0);
                let next_x = (curr_x + seg_w).min(text_x + total_bar_w);
                if next_x > curr_x {
                    let seg_rect = Rect::from_min_max(
                        Pos2::new(curr_x, row_rect.top()),
                        Pos2::new(next_x, row_rect.bottom()),
                    );
                    painter.rect_filled(seg_rect, Rounding::ZERO, resolve_tag_color(*tid, tags));
                    curr_x = next_x;
                }
            }
        }
    }

    // 5. Render Node Text in Crisp Black Directly Over the Bar (rounded coordinates, 12.0 font)
    let px = (text_x + 2.0).round();
    let py = (row_rect.top() + 2.0).round();
    painter.text(
        Pos2::new(px, py),
        egui::Align2::LEFT_TOP,
        &display_text,
        FontId::proportional(12.0),
        Color32::from_rgb(0, 0, 0),
    );

    // Hover tooltip
    row_resp.on_hover_ui(|ui| {
        ui.label(RichText::new(&node.name).strong().color(Color32::from_rgb(0, 0, 0)));
        ui.label(format!("Time: {}", format_pt_time(node.total_seconds)));
        ui.label(format!(
            "{:.1}% of root max",
            (node.total_seconds as f32 / max_seconds) * 100.0
        ));
    });

    // 6. Render Expanded Children with Proper Guideline Nesting
    if node.is_expanded && has_children {
        let child_indent = depth + 1;
        ancestor_has_more.push(!is_last_sibling);
        render_tree_nodes(
            ui,
            &mut node.children,
            tags,
            max_seconds,
            child_indent,
            selected_node_name,
            expanded_overrides,
            ancestor_has_more,
        );
        ancestor_has_more.pop();
    }
}
