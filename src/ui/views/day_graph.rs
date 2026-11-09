use crate::model::{ActivitySession, Tag};
use crate::ui::common::{format_pt_time, resolve_tag_color};
use crate::ui::theme::Theme;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use egui::{Color32, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct DayStat {
    pub date: NaiveDate,
    pub total_seconds: i64,
    pub tag_breakdown: BTreeMap<Option<i64>, i64>,
}

impl DayStat {
    pub fn compute_active_days(
        activities: &[ActivitySession],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<DayStat> {
        let mut day_map: BTreeMap<NaiveDate, DayStat> = BTreeMap::new();
        let mut curr = start_date;
        while curr <= end_date {
            day_map.insert(
                curr,
                DayStat {
                    date: curr,
                    total_seconds: 0,
                    tag_breakdown: BTreeMap::new(),
                },
            );
            match curr.succ_opt() {
                Some(next) => curr = next,
                None => break,
            }
        }

        for act in activities {
            if act.is_afk || act.duration <= 0 {
                continue;
            }
            if let Some(dt) = DateTime::from_timestamp(act.start_time, 0) {
                let local_date = dt.with_timezone(&Local).date_naive();
                if let Some(entry) = day_map.get_mut(&local_date) {
                    entry.total_seconds += act.duration;
                    *entry.tag_breakdown.entry(act.tag_id).or_insert(0) += act.duration;
                }
            }
        }

        day_map
            .into_values()
            .filter(|d| d.total_seconds > 0)
            .collect()
    }
}

pub fn render_day_minimap(
    ui: &mut Ui,
    active_days: &[DayStat],
    tags: &[Tag],
    hover_status: &mut Option<String>,
) {
    let theme = Theme::WIN32;
    let width = (ui.available_width() - 2.0).clamp(80.0, 100.0);
    let avail_height = ui.available_height().max(100.0);

    let biggest_total = active_days
        .iter()
        .map(|d| d.total_seconds)
        .max()
        .unwrap_or(0);

    // 3. Allocate full-height canvas painter
    let (response, painter) =
        ui.allocate_painter(Vec2::new(width, avail_height), egui::Sense::hover());
    let canvas_rect = response.rect;

    // Pure white background (#FFFFFF) with 1px border (#7A7A7A)
    painter.rect_filled(canvas_rect, Rounding::ZERO, Color32::from_rgb(255, 255, 255));
    painter.rect_stroke(
        canvas_rect,
        Rounding::ZERO,
        Stroke::new(1.0, theme.control_border),
    );

    let inner_rect = canvas_rect.shrink(1.0);
    let num_days = active_days.len();

    *hover_status = None;

    if num_days > 0 && biggest_total > 0 {
        let mut bar_h = (inner_rect.height() / num_days as f32).floor();
        if bar_h < 1.0 {
            bar_h = 1.0;
        }
        if bar_h > 20.0 {
            bar_h = 20.0;
        }
        let bar_w = inner_rect.width();

        for (i, day) in active_days.iter().enumerate() {
            let y_top = inner_rect.top() + (i as f32) * bar_h;
            let y_bottom = (y_top + bar_h).min(inner_rect.bottom());
            let mut bstart = inner_rect.left();

            // Stacked tag segments (1:1 with PT renderdaystat loop(j, MAXTAGS))
            for (&tid, &sec) in &day.tag_breakdown {
                let seg_w = ((sec as f32 / biggest_total as f32) * bar_w).max(1.0);
                let next_x = (bstart + seg_w).min(inner_rect.right());
                if next_x > bstart {
                    let seg_rect =
                        Rect::from_min_max(Pos2::new(bstart, y_top), Pos2::new(next_x, y_bottom));
                    let color = resolve_tag_color(tid, tags);
                    painter.rect_filled(seg_rect, Rounding::ZERO, color);
                    bstart = next_x;
                }
            }
        }

        // 4. Mouse hover hit-testing (1:1 with PT WM_MOUSEMOVE IDC_STATICHO)
        if let Some(pos) = response.hover_pos() {
            if inner_rect.contains(pos) {
                let hover_y = pos.y - inner_rect.top();
                let day_idx = (hover_y / bar_h) as usize;
                if day_idx < active_days.len() {
                    let day = &active_days[day_idx];
                    let time_str = format_pt_time(day.total_seconds);
                    *hover_status = Some(format!(
                        "{}-{}-{} -> {}",
                        day.date.year(),
                        day.date.month(),
                        day.date.day(),
                        time_str
                    ));
                }
            }
        }
    }
}
