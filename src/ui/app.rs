use crate::db::repository::Repository;
use crate::model::{ActivitySession, Tag};
use crate::tracker::SharedTrackerState;
use crate::ui::theme::{configure_win32_theme, Theme};
use crate::ui::views::statistics::{render_statistics_view, StatisticsControlsState};
use chrono::{Local, TimeZone};
use eframe::egui;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Instant;

pub struct ActLogApp {
    #[allow(dead_code)]
    state: Arc<SharedTrackerState>,
    db_path: String,

    statistics_state: StatisticsControlsState,

    // Data Cache
    activities: Vec<ActivitySession>,
    tags: Vec<Tag>,
    last_refresh: Instant,
    theme_configured: bool,

    screenshot_path: Option<String>,
    frame_count: u32,
}

fn save_color_image_to_bmp(image: &egui::ColorImage, path: &str) -> std::io::Result<()> {
    let width = image.width() as u32;
    let height = image.height() as u32;
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
            let pixel = image.pixels[(y * width + x) as usize];
            buf.push(pixel.b());
            buf.push(pixel.g());
            buf.push(pixel.r());
        }
        buf.extend(std::iter::repeat(0).take(pad));
    }
    std::fs::write(path, buf)
}

impl ActLogApp {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String, screenshot_path: Option<String>) -> Self {
        let mut app = Self {
            state,
            db_path,
            statistics_state: StatisticsControlsState::default(),
            activities: Vec::new(),
            tags: Vec::new(),
            last_refresh: Instant::now(),
            theme_configured: false,
            screenshot_path,
            frame_count: 0,
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
            self.statistics_state.mark_dirty();
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
}

impl eframe::App for ActLogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_configured {
            configure_win32_theme(ctx);
            self.theme_configured = true;
        }

        self.frame_count += 1;
        if self.screenshot_path.is_some() {
            if self.frame_count == 4 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            }
            ctx.input(|i| {
                for event in &i.raw.events {
                    if let egui::Event::Screenshot { image, .. } = event {
                        if let Some(ref path) = self.screenshot_path {
                            let _ = save_color_image_to_bmp(image, path);
                            std::process::exit(0);
                        }
                    }
                }
            });
            ctx.request_repaint();
        }

        // Auto-refresh data every 5 seconds
        if self.last_refresh.elapsed().as_secs() >= 5 {
            self.reload_data();
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));

        let theme = Theme::WIN32;
        let mut date_changed = false;
        let mut tag_assigned = None;
        let mut tag_renamed = None;

        // --- AUTHENTIC SINGLE-WINDOW 3-PANEL CLIENT AREA (1:1 with photo1 PT Statistics View) ---
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme.dialog_bg)
                    .inner_margin(egui::Margin::same(6.0)),
            )
            .show(ctx, |ui| {
                render_statistics_view(
                    ui,
                    &mut self.statistics_state,
                    &self.activities,
                    &self.tags,
                    &mut date_changed,
                    &mut tag_assigned,
                    &mut tag_renamed,
                );

                if date_changed {
                    self.reload_data();
                }

                if let Some((node_name, new_tag_id)) = tag_assigned {
                    if let Ok(conn) = Connection::open(&self.db_path) {
                        let _ = conn.execute(
                            "UPDATE activities SET tag_id = ?1 WHERE LOWER(app_name) = LOWER(?2)",
                            (new_tag_id, &node_name),
                        );
                        let _ = conn.execute(
                            "DELETE FROM rules WHERE LOWER(pattern) = LOWER(?1) AND match_field = 'app_name'",
                            [&node_name],
                        );
                        if let Some(tid) = new_tag_id {
                            let _ = conn.execute(
                                "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, 'app_name', ?2, 20)",
                                (&node_name, tid),
                            );
                        }
                    }
                    self.reload_data();
                }

                if let Some((tag_id, new_name)) = tag_renamed {
                    if let Ok(conn) = Connection::open(&self.db_path) {
                        let _ = conn.execute(
                            "UPDATE tags SET name = ?1 WHERE id = ?2",
                            (&new_name, tag_id),
                        );
                    }
                    self.reload_data();
                }
            });
    }
}
