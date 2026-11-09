use crate::db::repository::Repository;
use crate::model::{ActivitySession, Tag};
use crate::tracker::SharedTrackerState;
use crate::ui::theme::{dialog_container_style, white_panel_style};
use crate::ui::views::day_graph::DayMinimapCanvas;
use crate::ui::views::statistics::{
    render_sidebar, QuickSet, StatisticsControlsState, TreeCanvasProgram, ROW_HEIGHT,
};
use chrono::{Local, NaiveDate, TimeZone};
use iced::widget::canvas::Canvas;
use iced::widget::{container, row, scrollable};
use iced::{window, Element, Length, Subscription, Task};
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ExpandLevelChanged(u32),
    MinMinutesChanged(String),
    SubstringFilterChanged(String),
    StartDateChanged(String),
    EndDateChanged(String),
    QuickSetSelected(QuickSet),
    FilterOnTagToggled(bool),
    TagSelected(Option<i64>),
    NodeSelected(String),
    NodeToggled(String),
    ApplyTagToNode,
    RenameTag,
    DayMinimapHover(Option<String>),
    TreeScrolled { offset_y: f32, height: f32 },
    TriggerScreenshot,
    WindowIdResolved(Option<window::Id>),
    ScreenshotCaptured(window::Screenshot),
}

fn save_rgba_to_bmp(rgba: &[u8], width: u32, height: u32, path: &str) -> std::io::Result<()> {
    let row_size = (width * 3 + 3) & !3;
    let image_size = row_size * height;
    let file_size = 54 + image_size;

    let mut buf = Vec::with_capacity(file_size as usize);
    buf.extend_from_slice(b"BM");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(&[0, 0, 0, 0]);
    buf.extend_from_slice(&54u32.to_le_bytes());
    buf.extend_from_slice(&40u32.to_le_bytes());
    buf.extend_from_slice(&(width as i32).to_le_bytes());
    buf.extend_from_slice(&(height as i32).to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&24u16.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&image_size.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());

    let pad = (row_size - width * 3) as usize;
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if idx + 2 < rgba.len() {
                let r = rgba[idx];
                let g = rgba[idx + 1];
                let b = rgba[idx + 2];
                buf.push(b);
                buf.push(g);
                buf.push(r);
            }
        }
        buf.extend(std::iter::repeat(0).take(pad));
    }
    std::fs::write(path, buf)
}

pub struct ActLogApp {
    #[allow(dead_code)]
    state: Arc<SharedTrackerState>,
    db_path: String,
    pub statistics_state: StatisticsControlsState,
    pub activities: Vec<ActivitySession>,
    pub tags: Vec<Tag>,
    last_refresh: Instant,
    pub screenshot_path: Option<String>,
}

impl ActLogApp {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String, screenshot_path: Option<String>) -> (Self, Task<Message>) {
        let mut app = Self {
            state,
            db_path,
            statistics_state: StatisticsControlsState::default(),
            activities: Vec::new(),
            tags: Vec::new(),
            last_refresh: Instant::now(),
            screenshot_path: screenshot_path.clone(),
        };
        app.reload_data();

        let initial_task = if screenshot_path.is_some() {
            Task::perform(
                async {
                    std::thread::sleep(std::time::Duration::from_millis(800));
                },
                |_| Message::TriggerScreenshot,
            )
        } else {
            Task::none()
        };

        (app, initial_task)
    }

    pub fn reload_data(&mut self) {
        if let Ok(conn) = Connection::open(&self.db_path) {
            self.tags = Repository::get_all_tags(&conn).unwrap_or_default();

            let (start_ts, end_ts) = self.calculate_timestamps();
            self.activities =
                Repository::get_day_activities(&conn, start_ts, end_ts).unwrap_or_default();
            self.statistics_state.mark_dirty();
            self.statistics_state.sync_cache(&self.activities);
        }
        self.last_refresh = Instant::now();
    }

    fn calculate_timestamps(&self) -> (i64, i64) {
        let s = Local
            .from_local_datetime(&self.statistics_state.start_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        let next_date = self
            .statistics_state
            .end_date
            .succ_opt()
            .unwrap_or(self.statistics_state.end_date);
        let e = Local
            .from_local_datetime(&next_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        (s, e)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.last_refresh.elapsed().as_secs() >= 5 {
                    self.reload_data();
                }
            }
            Message::ExpandLevelChanged(lvl) => {
                self.statistics_state.expand_level = lvl;
                self.statistics_state.mark_dirty();
                self.statistics_state.sync_cache(&self.activities);
            }
            Message::MinMinutesChanged(s) => {
                self.statistics_state.filter_min_minutes_str = s.clone();
                if let Ok(mins) = s.parse::<i64>() {
                    self.statistics_state.filter_min_minutes = mins.max(0);
                    self.statistics_state.mark_dirty();
                    self.statistics_state.sync_cache(&self.activities);
                }
            }
            Message::SubstringFilterChanged(s) => {
                self.statistics_state.filter_substring = s;
                self.statistics_state.mark_dirty();
                self.statistics_state.sync_cache(&self.activities);
            }
            Message::StartDateChanged(s) => {
                self.statistics_state.start_date_str = s.clone();
                if let Ok(d) = NaiveDate::parse_from_str(&s, "%m/%d/%Y") {
                    self.statistics_state.start_date = d;
                    self.reload_data();
                }
            }
            Message::EndDateChanged(s) => {
                self.statistics_state.end_date_str = s.clone();
                if let Ok(d) = NaiveDate::parse_from_str(&s, "%m/%d/%Y") {
                    self.statistics_state.end_date = d;
                    self.reload_data();
                }
            }
            Message::QuickSetSelected(qs) => {
                self.statistics_state.quick_set = qs;
                let today = Local::now().date_naive();
                let (s, e) = qs.compute_range(today);
                self.statistics_state.start_date = s;
                self.statistics_state.end_date = e;
                self.statistics_state.sync_date_strings();
                self.reload_data();
            }
            Message::FilterOnTagToggled(val) => {
                self.statistics_state.filter_on_tag = val;
                self.statistics_state.mark_dirty();
                self.statistics_state.sync_cache(&self.activities);
            }
            Message::TagSelected(tid) => {
                self.statistics_state.selected_tag_id = tid;
                if self.statistics_state.filter_on_tag {
                    self.statistics_state.mark_dirty();
                    self.statistics_state.sync_cache(&self.activities);
                }
            }
            Message::NodeSelected(name) => {
                self.statistics_state.selected_node_name = Some(name);
            }
            Message::NodeToggled(name) => {
                let current = *self.statistics_state.expanded_overrides.get(&name).unwrap_or(&true);
                self.statistics_state.expanded_overrides.insert(name, !current);
                self.statistics_state.mark_dirty();
                self.statistics_state.sync_cache(&self.activities);
            }
            Message::ApplyTagToNode => {
                if let Some(ref node_name) = self.statistics_state.selected_node_name {
                    let new_tag_id = self.statistics_state.selected_tag_id;
                    if let Ok(conn) = Connection::open(&self.db_path) {
                        let _ = conn.execute(
                            "UPDATE activities SET tag_id = ?1 WHERE LOWER(app_name) = LOWER(?2)",
                            (new_tag_id, node_name),
                        );
                        let _ = conn.execute(
                            "DELETE FROM rules WHERE LOWER(pattern) = LOWER(?1) AND match_field = 'app_name'",
                            [node_name],
                        );
                        if let Some(tid) = new_tag_id {
                            let _ = conn.execute(
                                "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, 'app_name', ?2, 20)",
                                (node_name, tid),
                            );
                        }
                    }
                    self.reload_data();
                }
            }
            Message::RenameTag => {
                // Future modal/input dialog
            }
            Message::DayMinimapHover(status) => {
                self.statistics_state.hover_status = status;
            }
            Message::TreeScrolled { offset_y, height } => {
                self.statistics_state.tree_scroll_y = offset_y;
                self.statistics_state.tree_viewport_height = height;
            }
            Message::TriggerScreenshot => {
                return window::latest().map(Message::WindowIdResolved);
            }
            Message::WindowIdResolved(Some(id)) => {
                return window::screenshot(id).map(Message::ScreenshotCaptured);
            }
            Message::WindowIdResolved(None) => {}
            Message::ScreenshotCaptured(screenshot) => {
                if let Some(ref path) = self.screenshot_path {
                    let _ = save_rgba_to_bmp(
                        &screenshot.rgba,
                        screenshot.size.width,
                        screenshot.size.height,
                        path,
                    );
                    std::process::exit(0);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // 1. Left Sidebar (~185px)
        let sidebar = render_sidebar(
            &self.statistics_state,
            &self.tags,
            Message::ExpandLevelChanged,
            Message::MinMinutesChanged,
            Message::SubstringFilterChanged,
            Message::StartDateChanged,
            Message::EndDateChanged,
            Message::QuickSetSelected,
            Message::FilterOnTagToggled,
            Message::ApplyTagToNode,
            Message::RenameTag,
            Message::TagSelected,
        );

        // 2. Center TreeView
        let rows_count = self.statistics_state.cached_flat_rows.len();
        let total_canvas_h = ((rows_count as f32) * ROW_HEIGHT + 20.0).max(100.0);

        let tree_canvas = Canvas::new(TreeCanvasProgram {
            rows: &self.statistics_state.cached_flat_rows,
            tags: &self.tags,
            max_seconds: self.statistics_state.cached_max_seconds,
            selected_node: self.statistics_state.selected_node_name.as_deref(),
            scroll_y: self.statistics_state.tree_scroll_y,
            viewport_height: self.statistics_state.tree_viewport_height,
            on_toggle: Box::new(Message::NodeToggled),
            on_select: Box::new(Message::NodeSelected),
        })
        .width(Length::Fill)
        .height(Length::Fixed(total_canvas_h));

        let tree_scrollable = scrollable(tree_canvas)
            .on_scroll(|vp| Message::TreeScrolled {
                offset_y: vp.absolute_offset().y,
                height: vp.bounds().height,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        let center_panel = container(tree_scrollable)
            .style(white_panel_style)
            .width(Length::Fill)
            .height(Length::Fill);

        // 3. Right Day Minimap (~90px)
        let minimap_canvas = Canvas::new(DayMinimapCanvas {
            active_days: &self.statistics_state.cached_day_stats,
            tags: &self.tags,
            on_hover: Box::new(Message::DayMinimapHover),
        })
        .width(Length::Fixed(90.0))
        .height(Length::Fill);

        let right_panel = container(minimap_canvas)
            .style(white_panel_style)
            .width(Length::Fixed(90.0))
            .height(Length::Fill);

        // Client Area: 3-panel horizontal layout
        let content = row![
            sidebar,
            center_panel,
            right_panel,
        ]
        .spacing(6)
        .padding(6)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .style(dialog_container_style)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick)
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }
}
