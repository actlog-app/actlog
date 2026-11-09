use crate::model::{ActivitySession, Tag};
use crate::ui::common::{format_pt_time, resolve_tag_color};
use crate::ui::theme::Win32Theme;
use chrono::{DateTime, Local, NaiveDate};
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program};
use iced::{Point, Rectangle, Renderer, Size};
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

pub struct DayMinimapCanvas<'a, Message> {
    pub active_days: &'a [DayStat],
    pub tags: &'a [Tag],
    pub on_hover: Box<dyn Fn(Option<String>) -> Message + 'a>,
}

impl<'a, Message: Clone + 'static> Program<Message> for DayMinimapCanvas<'a, Message> {
    type State = Option<usize>;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, Win32Theme::WHITE_PANEL);

        let num_days = self.active_days.len();
        let biggest_total = self
            .active_days
            .iter()
            .map(|d| d.total_seconds)
            .max()
            .unwrap_or(0);

        if num_days > 0 && biggest_total > 0 {
            let inner_w = (bounds.width - 2.0).max(1.0);
            let inner_h = (bounds.height - 2.0).max(1.0);
            let mut bar_h = (inner_h / num_days as f32).floor();
            if bar_h < 1.0 {
                bar_h = 1.0;
            }
            if bar_h > 20.0 {
                bar_h = 20.0;
            }

            for (i, day) in self.active_days.iter().enumerate() {
                let y = 1.0 + (i as f32) * bar_h;
                if y + bar_h > bounds.height - 1.0 {
                    break;
                }

                let total_ratio = (day.total_seconds as f32 / biggest_total as f32).clamp(0.0, 1.0);
                let day_bar_w = (inner_w * total_ratio).max(1.0);

                let mut cur_x = 1.0;
                for (&tag_id, &tag_sec) in &day.tag_breakdown {
                    if tag_sec <= 0 {
                        continue;
                    }
                    let seg_w = (day_bar_w * (tag_sec as f32 / day.total_seconds as f32)).max(1.0);
                    let color = resolve_tag_color(tag_id, self.tags);
                    let rect = Path::rectangle(Point::new(cur_x, y), Size::new(seg_w, bar_h.max(1.0) - 0.5));
                    frame.fill(&rect, color);
                    cur_x += seg_w;
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        last_hovered: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new_idx = if let Some(pos) = cursor.position_in(bounds) {
                    let num_days = self.active_days.len();
                    if num_days == 0 {
                        None
                    } else {
                        let inner_h = (bounds.height - 2.0).max(1.0);
                        let mut bar_h = (inner_h / num_days as f32).floor();
                        if bar_h < 1.0 {
                            bar_h = 1.0;
                        }
                        if bar_h > 20.0 {
                            bar_h = 20.0;
                        }

                        let rel_y = pos.y - 1.0;
                        if rel_y >= 0.0 {
                            let idx = (rel_y / bar_h).floor() as usize;
                            if idx < self.active_days.len() {
                                Some(idx)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                } else {
                    None
                };

                // Deduplicate: only fire if hovered day changed
                if *last_hovered != new_idx {
                    *last_hovered = new_idx;
                    let msg = new_idx.map(|idx| {
                        let day = &self.active_days[idx];
                        format!(
                            "{} -> {}",
                            day.date.format("%Y-%m-%d"),
                            format_pt_time(day.total_seconds)
                        )
                    });
                    return Some(canvas::Action::publish((self.on_hover)(msg)));
                }
                None
            }
            _ => None,
        }
    }
}
