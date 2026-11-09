use crate::model::{ActivitySession, Tag};
use crate::ui::common::{format_pt_time, resolve_tag_color};
use crate::ui::theme::{
    groupbox_style, win32_button_style, win32_pick_list_style, win32_slider_style,
    win32_text_input_style, Win32Theme,
};
use crate::ui::views::day_graph::DayStat;
use chrono::{Datelike, Duration, Local, NaiveDate};
use iced::alignment::{Horizontal, Vertical};
use iced::border::Radius;
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, LineDash, Path, Program, Stroke, Text};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, slider, text, text_input,
};
use iced::{Background, Border, Element, Length, Point, Rectangle, Renderer, Size};
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

impl std::fmt::Display for QuickSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
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

#[derive(Clone, Debug)]
pub struct FlatRow {
    pub name: String,
    pub display_name: String,
    pub total_seconds: i64,
    pub tag_id: Option<i64>,
    pub tag_breakdown: BTreeMap<Option<i64>, i64>,
    pub depth: usize,
    pub has_children: bool,
    pub is_expanded: bool,
    pub is_last_sibling: bool,
    pub ancestor_is_last: Vec<bool>,
}

impl FlatRow {
    pub fn flatten(
        nodes: &[ActivityNode],
        depth: usize,
        ancestor_is_last: &mut Vec<bool>,
        out: &mut Vec<FlatRow>,
    ) {
        let count = nodes.len();
        for (i, node) in nodes.iter().enumerate() {
            let is_last = i == count - 1;
            let has_children = !node.children.is_empty();

            out.push(FlatRow {
                name: node.name.clone(),
                display_name: node.name.clone(),
                total_seconds: node.total_seconds,
                tag_id: node.tag_id,
                tag_breakdown: node.tag_breakdown.clone(),
                depth,
                has_children,
                is_expanded: node.is_expanded,
                is_last_sibling: is_last,
                ancestor_is_last: ancestor_is_last.clone(),
            });

            if has_children && node.is_expanded {
                ancestor_is_last.push(is_last);
                Self::flatten(&node.children, depth + 1, ancestor_is_last, out);
                ancestor_is_last.pop();
            }
        }
    }
}

pub const ROW_HEIGHT: f32 = 18.0;

pub struct TreeCanvasProgram<'a, Message> {
    pub rows: &'a [FlatRow],
    pub tags: &'a [Tag],
    pub max_seconds: f32,
    pub selected_node: Option<&'a str>,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub on_toggle: Box<dyn Fn(String) -> Message + 'a>,
    pub on_select: Box<dyn Fn(String) -> Message + 'a>,
}

impl<'a, Message: Clone + 'static> Program<Message> for TreeCanvasProgram<'a, Message> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Solid white background
        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, Win32Theme::WHITE_PANEL);

        let dotted_stroke = Stroke {
            style: canvas::Style::Solid(Win32Theme::GROUPBOX_BORDER),
            width: 1.0,
            line_dash: LineDash {
                segments: &[1.0, 1.0],
                offset: 0,
            },
            ..Stroke::default()
        };

        let start_y = self.scroll_y.max(0.0);
        let end_y = start_y + self.viewport_height.max(500.0);
        let first_idx = ((start_y / ROW_HEIGHT).floor() as usize).saturating_sub(1);
        let last_idx = (((end_y / ROW_HEIGHT).ceil() as usize) + 2).min(self.rows.len());

        for i in first_idx..last_idx {
            let row = &self.rows[i];
            let y = 2.0 + (i as f32) * ROW_HEIGHT;

            let is_root = row.depth == 0;
            let is_selected = self.selected_node == Some(&row.name) || (is_root && self.selected_node.is_none());

            // 1. Selection background if selected
            if is_selected {
                let sel_rect = Path::rectangle(
                    Point::new(1.0, y),
                    Size::new(bounds.width - 2.0, ROW_HEIGHT),
                );
                frame.fill(&sel_rect, Win32Theme::SELECTION_BG);
                frame.stroke(
                    &sel_rect,
                    Stroke::default()
                        .with_color(Win32Theme::SELECTION_BORDER)
                        .with_width(1.0),
                );
            }

            // 2. Dotted Guidelines
            if !is_root {
                for (d, &is_last) in row.ancestor_is_last.iter().enumerate().skip(1) {
                    if !is_last {
                        let line_x = (d as f32) * 16.0 + 8.0;
                        let line = Path::line(
                            Point::new(line_x, y),
                            Point::new(line_x, y + ROW_HEIGHT),
                        );
                        frame.stroke(&line, dotted_stroke);
                    }
                }

                let cur_x = (row.depth as f32) * 16.0 + 8.0;
                let v_bottom = if row.is_last_sibling { y + 9.0 } else { y + ROW_HEIGHT };
                let v_line = Path::line(
                    Point::new(cur_x, y),
                    Point::new(cur_x, v_bottom),
                );
                frame.stroke(&v_line, dotted_stroke);

                let h_line = Path::line(
                    Point::new(cur_x, y + 9.0),
                    Point::new(cur_x + 8.0, y + 9.0),
                );
                frame.stroke(&h_line, dotted_stroke);
            }

            // 3. Toggle button box
            let toggle_x = (row.depth as f32) * 16.0 + (if is_root { 4.0 } else { 12.0 });
            if row.has_children {
                let box_rect = Path::rectangle(
                    Point::new(toggle_x, y + 4.5),
                    Size::new(9.0, 9.0),
                );
                frame.fill(&box_rect, Win32Theme::WHITE_PANEL);
                frame.stroke(
                    &box_rect,
                    Stroke::default()
                        .with_color(Win32Theme::PANEL_BORDER)
                        .with_width(1.0),
                );

                // Minus line: horizontal tick
                let h_tick = Path::line(
                    Point::new(toggle_x + 2.0, y + 9.0),
                    Point::new(toggle_x + 7.0, y + 9.0),
                );
                frame.stroke(&h_tick, Stroke::default().with_color(Win32Theme::TEXT_BLACK).with_width(1.0));

                if !row.is_expanded {
                    // Plus line: vertical tick
                    let v_tick = Path::line(
                        Point::new(toggle_x + 4.5, y + 6.5),
                        Point::new(toggle_x + 4.5, y + 11.5),
                    );
                    frame.stroke(&v_tick, Stroke::default().with_color(Win32Theme::TEXT_BLACK).with_width(1.0));
                }
            }

            // 4. In-row colored bar (rendered directly behind text)
            let text_x = toggle_x + 14.0;
            if !is_root && row.total_seconds > 0 && self.max_seconds > 0.0 {
                let avail_bar_w = (bounds.width - text_x - 30.0).max(10.0);
                let bar_w = ((row.total_seconds as f32 / self.max_seconds) * avail_bar_w).clamp(2.0, avail_bar_w);

                let mut cur_bar_x = text_x;
                if row.tag_breakdown.len() > 1 {
                    for (&tag_id, &sec) in &row.tag_breakdown {
                        if sec <= 0 {
                            continue;
                        }
                        let seg_w = (bar_w * (sec as f32 / row.total_seconds as f32)).max(1.0);
                        let color = resolve_tag_color(tag_id, self.tags);
                        let seg_rect = Path::rectangle(
                            Point::new(cur_bar_x, y + 1.0),
                            Size::new(seg_w, ROW_HEIGHT - 2.0),
                        );
                        frame.fill(&seg_rect, color);
                        cur_bar_x += seg_w;
                    }
                } else {
                    let color = resolve_tag_color(row.tag_id, self.tags);
                    let bar_rect = Path::rectangle(
                        Point::new(cur_bar_x, y + 1.0),
                        Size::new(bar_w, ROW_HEIGHT - 2.0),
                    );
                    frame.fill(&bar_rect, color);
                }
            }

            // 5. Crisp Black Text
            let display_str = if is_root {
                format!("{} - (root)", format_pt_time(row.total_seconds))
            } else {
                format!("{} - {}", format_pt_time(row.total_seconds), row.display_name)
            };

            frame.fill_text(Text {
                content: display_str,
                position: Point::new(text_x + 2.0, y + 2.0),
                color: Win32Theme::TEXT_BLACK,
                size: 11.0.into(),
                line_height: iced::widget::text::LineHeight::default(),
                font: iced::Font::with_name("Segoe UI"),
                align_x: Horizontal::Left.into(),
                align_y: Vertical::Top.into(),
                shaping: iced::widget::text::Shaping::Basic,
                max_width: f32::INFINITY,
            });
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if let Some(pos) = cursor.position_in(bounds) {
                let rel_y = pos.y - 2.0;
                if rel_y >= 0.0 {
                    let idx = (rel_y / ROW_HEIGHT).floor() as usize;
                    if idx < self.rows.len() {
                        let row = &self.rows[idx];
                        let is_root = row.depth == 0;
                        let toggle_x = (row.depth as f32) * 16.0 + (if is_root { 4.0 } else { 12.0 });

                        // Hit-test square toggle box
                        if row.has_children && pos.x >= toggle_x - 3.0 && pos.x <= toggle_x + 13.0 {
                            return Some(canvas::Action::publish((self.on_toggle)(row.name.clone())));
                        }

                        // Otherwise select node row
                        return Some(canvas::Action::publish((self.on_select)(row.name.clone())));
                    }
                }
            }
        }
        None
    }
}

pub struct StatisticsControlsState {
    pub expand_level: u32,
    pub filter_min_minutes: i64,
    pub filter_min_minutes_str: String,
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

    // Performance & Virtualization
    pub tree_scroll_y: f32,
    pub tree_viewport_height: f32,
    pub cached_tree: Vec<ActivityNode>,
    pub cached_flat_rows: Vec<FlatRow>,
    pub cached_max_seconds: f32,
    pub cached_day_stats: Vec<DayStat>,
    pub is_dirty: bool,
}

impl Default for StatisticsControlsState {
    fn default() -> Self {
        let today = Local::now().date_naive();
        let mut s = Self {
            expand_level: 2,
            filter_min_minutes: 0,
            filter_min_minutes_str: "0".to_string(),
            filter_substring: String::new(),
            quick_set: QuickSet::Today,
            start_date: today,
            end_date: today,
            start_date_str: format!("{}/{}/{}", today.month(), today.day(), today.year()),
            end_date_str: format!("{}/{}/{}", today.month(), today.day(), today.year()),
            filter_on_tag: false,
            selected_tag_id: None,
            selected_node_name: None,
            renaming_tag_id: None,
            rename_buffer: String::new(),
            hover_status: None,
            expanded_overrides: std::collections::HashMap::new(),
            tree_scroll_y: 0.0,
            tree_viewport_height: 560.0,
            cached_tree: Vec::new(),
            cached_flat_rows: Vec::new(),
            cached_max_seconds: 1.0,
            cached_day_stats: Vec::new(),
            is_dirty: true,
        };
        s.sync_date_strings();
        s
    }
}

impl StatisticsControlsState {
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
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

            self.cached_max_seconds = if !self.cached_tree.is_empty() && !self.cached_tree[0].children.is_empty() {
                self.cached_tree[0].children[0].total_seconds.max(1) as f32
            } else if !self.cached_tree.is_empty() {
                self.cached_tree[0].total_seconds.max(1) as f32
            } else {
                1.0
            };

            let mut flat = Vec::new();
            let mut ancestors = Vec::new();
            FlatRow::flatten(&self.cached_tree, 0, &mut ancestors, &mut flat);
            self.cached_flat_rows = flat;

            self.cached_day_stats = DayStat::compute_active_days(
                activities,
                self.start_date,
                self.end_date,
            );

            self.is_dirty = false;
        }
    }
}

pub fn render_sidebar<'a, Message: Clone + 'static>(
    state: &'a StatisticsControlsState,
    tags: &'a [Tag],
    on_expand_level: impl Fn(u32) -> Message + 'static,
    on_min_minutes: impl Fn(String) -> Message + 'static,
    on_substring: impl Fn(String) -> Message + 'static,
    on_start_date: impl Fn(String) -> Message + 'static,
    on_end_date: impl Fn(String) -> Message + 'static,
    on_quick_set: impl Fn(QuickSet) -> Message + 'static,
    on_filter_tag_toggle: impl Fn(bool) -> Message + 'static,
    on_apply_tag: Message,
    on_rename_tag: Message,
    on_select_tag: impl Fn(Option<i64>) -> Message + 'static,
) -> Element<'a, Message> {
    // 1. Expanded Level
    let level_slider = row![
        text("Expanded Level:").size(11).color(Win32Theme::TEXT_BLACK),
        slider(1..=5, state.expand_level, on_expand_level)
            .style(win32_slider_style)
            .width(Length::Fill),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // 2. Filter < Minutes
    let min_filter = column![
        text("Filter < Minutes:").size(11).color(Win32Theme::TEXT_BLACK),
        text_input("", &state.filter_min_minutes_str)
            .on_input(on_min_minutes)
            .style(win32_text_input_style)
            .padding(2)
            .size(11),
    ]
    .spacing(2);

    // 3. Filter on sub-strings
    let sub_filter = column![
        text("Filter on sub-strings:").size(11).color(Win32Theme::TEXT_BLACK),
        text_input("", &state.filter_substring)
            .on_input(on_substring)
            .style(win32_text_input_style)
            .padding(2)
            .size(11),
    ]
    .spacing(2);

    // 4. Date Range GroupBox
    let date_range_content = column![
        text("Date Range").size(10).color(Win32Theme::TEXT_MUTED),
        row![
            text_input("", &state.start_date_str)
                .on_input(on_start_date)
                .style(win32_text_input_style)
                .padding(2)
                .size(11)
                .width(Length::Fill),
            button(text("📅").size(10))
                .style(win32_button_style)
                .padding(2),
        ]
        .spacing(2),
        row![
            text_input("", &state.end_date_str)
                .on_input(on_end_date)
                .style(win32_text_input_style)
                .padding(2)
                .size(11)
                .width(Length::Fill),
            button(text("📅").size(10))
                .style(win32_button_style)
                .padding(2),
        ]
        .spacing(2),
        column![
            text("Quick Set:").size(10).color(Win32Theme::TEXT_BLACK),
            pick_list(QuickSet::ALL, Some(state.quick_set), on_quick_set)
                .style(win32_pick_list_style)
                .padding(2)
                .text_size(11)
                .width(Length::Fill),
        ]
        .spacing(1),
    ]
    .spacing(4)
    .padding(4);

    let date_range_box = container(date_range_content)
        .style(groupbox_style)
        .width(Length::Fill);

    // 5. Tagging GroupBox
    let filter_tag_cb = checkbox(state.filter_on_tag)
        .label("Filter on tag")
        .on_toggle(on_filter_tag_toggle)
        .size(12)
        .text_size(11);

    let btn_apply = button(text("Apply Tag to Node").size(10))
        .on_press(on_apply_tag)
        .style(win32_button_style)
        .padding(2)
        .width(Length::Fill);

    let btn_rename = button(text("Rename Tag").size(10))
        .on_press(on_rename_tag)
        .style(win32_button_style)
        .padding(2)
        .width(Length::Fill);

    // 15 Canonical Tags List in scrollable/list box
    let mut tag_items = column![].spacing(1);

    // Default UNTAGGED item
    let is_untagged_selected = state.selected_tag_id.is_none();
    let untagged_row = button(
        row![
            container(row![])
                .width(9)
                .height(9)
                .style(|_t| container::Style {
                    text_color: None,
                    background: Some(Background::Color(Win32Theme::UNTAGGED_GRAY)),
                    border: Border {
                        color: Win32Theme::PANEL_BORDER,
                        width: 1.0,
                        radius: Radius::from(0.0),
                    },
                    shadow: iced::Shadow::default(),
                    snap: false,
                }),
            text("UNTAGGED").size(10).color(Win32Theme::TEXT_BLACK),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .on_press(on_select_tag(None))
    .style(move |_t, _s| button::Style {
        background: Some(Background::Color(if is_untagged_selected {
            Win32Theme::SELECTION_BG
        } else {
            Win32Theme::WHITE_PANEL
        })),
        text_color: Win32Theme::TEXT_BLACK,
        border: Border::default(),
        shadow: iced::Shadow::default(),
        snap: false,
    })
    .padding(1)
    .width(Length::Fill);
    tag_items = tag_items.push(untagged_row);

    // 14 Canonical defined tags
    for t in tags {
        if t.name.eq_ignore_ascii_case("untagged") {
            continue;
        }
        let is_sel = state.selected_tag_id == t.id;
        let t_color = crate::ui::common::parse_hex_color(&t.color);
        let tag_id = t.id;
        let t_row = button(
            row![
                container(row![])
                    .width(9)
                    .height(9)
                    .style(move |_th| container::Style {
                        text_color: None,
                        background: Some(Background::Color(t_color)),
                        border: Border {
                            color: Win32Theme::PANEL_BORDER,
                            width: 1.0,
                            radius: Radius::from(0.0),
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    }),
                text(&t.name).size(10).color(Win32Theme::TEXT_BLACK),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        )
        .on_press(on_select_tag(tag_id))
        .style(move |_th, _s| button::Style {
            background: Some(Background::Color(if is_sel {
                Win32Theme::SELECTION_BG
            } else {
                Win32Theme::WHITE_PANEL
            })),
            text_color: Win32Theme::TEXT_BLACK,
            border: Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        })
        .padding(1)
        .width(Length::Fill);

        tag_items = tag_items.push(t_row);
    }

    let tag_list_box = container(scrollable(tag_items).height(Length::Fixed(160.0)))
        .style(crate::ui::theme::white_panel_style)
        .width(Length::Fill);

    let tagging_content = column![
        text("Tagging").size(10).color(Win32Theme::TEXT_MUTED),
        filter_tag_cb,
        btn_apply,
        btn_rename,
        tag_list_box,
    ]
    .spacing(4)
    .padding(4);

    let tagging_box = container(tagging_content)
        .style(groupbox_style)
        .width(Length::Fill);

    // 6. Bottom dynamic status label
    let status_str = state
        .hover_status
        .as_deref()
        .unwrap_or("Hover over graph...");
    let status_label = text(status_str)
        .size(10)
        .color(Win32Theme::TEXT_MUTED);

    column![
        level_slider,
        min_filter,
        sub_filter,
        date_range_box,
        tagging_box,
        status_label,
    ]
    .spacing(6)
    .width(185)
    .into()
}
