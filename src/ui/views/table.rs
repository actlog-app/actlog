use crate::db::models::{ActivitySession, DaySummary, Tag};
use crate::ui::views::summary::format_duration;
use crate::ui::views::timeline::{parse_hex_color, resolve_tag_color};
use egui::{Color32, RichText, Rounding, ScrollArea, Ui};

pub fn render_activity_breakdown(
    ui: &mut Ui,
    activities: &[ActivitySession],
    summary: Option<&DaySummary>,
    tags: &[Tag],
) {
    ui.columns(2, |columns| {
        // Left Column: Category Totals
        columns[0].vertical(|ui| {
            ui.label(RichText::new("CATEGORIES").strong().color(Color32::from_rgb(180, 180, 180)));
            ui.add_space(4.0);

            if let Some(sum) = summary {
                let total_active = sum.total_active_seconds.max(1) as f32;
                ScrollArea::vertical()
                    .id_salt("categories_scroll")
                    .max_height(350.0)
                    .show(ui, |ui| {
                        for cat in &sum.categories {
                            let ratio = (cat.total_seconds as f32 / total_active) * 100.0;
                            let color = parse_hex_color(&cat.color);

                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(10.0, 10.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, Rounding::same(5.0), color);
                                ui.label(RichText::new(&cat.tag_name).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.1}% ({})", ratio, format_duration(cat.total_seconds))).color(Color32::from_rgb(160, 160, 160)));
                                });
                            });
                            ui.separator();
                        }
                    });
            } else {
                ui.label("No data for date");
            }
        });

        // Right Column: Activities / Applications Table
        columns[1].vertical(|ui| {
            ui.label(RichText::new("ACTIVITY LOG").strong().color(Color32::from_rgb(180, 180, 180)));
            ui.add_space(4.0);

            ScrollArea::vertical()
                .id_salt("activities_scroll")
                .max_height(350.0)
                .show(ui, |ui| {
                    if activities.is_empty() {
                        ui.label(RichText::new("No activities recorded on this date.").color(Color32::GRAY));
                    } else {
                        for act in activities.iter().rev() {
                            if act.is_afk {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                let tag_color = resolve_tag_color(act.tag_id, tags);
                                let (pill_rect, _) = ui.allocate_exact_size(egui::Vec2::new(6.0, 14.0), egui::Sense::hover());
                                ui.painter().rect_filled(pill_rect, Rounding::same(3.0), tag_color);

                                ui.vertical(|ui| {
                                    let title_display = if let Some(ref domain) = act.domain {
                                        format!("{} ({})", domain, act.app_name)
                                    } else {
                                        act.app_name.clone()
                                    };
                                    ui.label(RichText::new(title_display).strong().size(13.0));

                                    let subtitle = if act.window_title.is_empty() {
                                        "—"
                                    } else {
                                        &act.window_title
                                    };
                                    ui.label(RichText::new(truncate_str(subtitle, 50)).size(11.0).color(Color32::from_rgb(140, 140, 140)));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format_duration(act.duration)).strong());
                                });
                            });
                            ui.separator();
                        }
                    }
                });
        });
    });
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let mut truncated: String = s.chars().take(max_len).collect();
        truncated.push('…');
        truncated
    } else {
        s.to_string()
    }
}
