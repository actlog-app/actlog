use crate::db::models::{ActivitySession, Tag};
use crate::ui::theme::Theme;
use crate::ui::views::timeline::parse_hex_color;
use crate::ui::views::tree::format_pt_time;
use egui::{Color32, Rect, RichText, Rounding, ScrollArea, Stroke, Ui, Vec2};
use std::collections::{BTreeMap, HashSet};

pub fn render_apps_and_categories(
    ui: &mut Ui,
    activities: &[ActivitySession],
    tags: &[Tag],
    selected_category: &mut Option<String>,
    search_query: &mut String,
    expanded_apps: &mut HashSet<String>,
    on_tag_assigned: &mut Option<(String, i64)>, // (app_name, new_tag_id)
) {
    let theme = Theme::DARK;
    let total_active_sec: i64 = activities
        .iter()
        .filter(|a| !a.is_afk)
        .map(|a| a.duration)
        .sum();

    ui.columns(2, |cols| {
        // Left Column: Categories (fixed width allocation)
        cols[0].set_max_width(220.0);
        cols[0].vertical(|ui| {
            let frame = egui::Frame::group(ui.style())
                .fill(theme.surface)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::symmetric(12.0, 10.0));

            frame.show(ui, |ui| {
                ui.label(RichText::new("CATEGORIES").size(10.5).strong().color(theme.text_dim));
                ui.add_space(6.0);

                // "All Categories" row
                let is_all_sel = selected_category.is_none();
                let all_btn = ui.selectable_label(
                    is_all_sel,
                    RichText::new("All Categories").size(12.0).strong(),
                );
                if all_btn.clicked() {
                    *selected_category = None;
                }

                ui.separator();

                // Compute category sums
                let mut cat_sums: BTreeMap<String, (i64, String)> = BTreeMap::new();
                for act in activities {
                    if act.is_afk {
                        continue;
                    }
                    let (cat_name, cat_color) = resolve_tag_info(act.tag_id, tags);
                    let entry = cat_sums.entry(cat_name).or_insert((0, cat_color));
                    entry.0 += act.duration;
                }

                ScrollArea::vertical()
                    .id_salt("modern_categories_scroll")
                    .show(ui, |ui| {
                        for (cat_name, (secs, color_hex)) in cat_sums {
                            let is_sel = selected_category.as_deref() == Some(&cat_name);
                            let color = parse_hex_color(&color_hex);
                            let ratio = (secs as f32 / total_active_sec.max(1) as f32) * 100.0;

                            ui.horizontal(|ui| {
                                let (chip, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(chip.center(), 4.0, color);

                                let label = RichText::new(&cat_name).size(12.0);
                                if ui.selectable_label(is_sel, label).clicked() {
                                    *selected_category = if is_sel { None } else { Some(cat_name.clone()) };
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{:.0}%", ratio))
                                            .size(11.0)
                                            .monospace()
                                            .color(theme.text_dim),
                                    );
                                });
                            });
                        }
                    });
            });
        });

        // Right Column: Applications & Drill-down
        cols[1].vertical(|ui| {
            let frame = egui::Frame::group(ui.style())
                .fill(theme.surface)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::symmetric(14.0, 10.0));

            frame.show(ui, |ui| {
                // Header + Search bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new("APPLICATIONS & TASKS").size(10.5).strong().color(theme.text_dim));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let search = egui::TextEdit::singleline(search_query)
                            .hint_text("🔍 Search apps, tabs, documents...")
                            .desired_width(220.0);
                        ui.add(search);
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Group activities into App -> Window Titles
                let app_groups = build_app_groups(activities, tags, selected_category.as_deref(), search_query);

                ScrollArea::vertical()
                    .id_salt("modern_apps_scroll")
                    .show(ui, |ui| {
                        if app_groups.is_empty() {
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("No activities match search or selected category.").color(theme.text_dim).size(12.5));
                            });
                        } else {
                            for app in app_groups {
                                render_app_row(
                                    ui,
                                    &app,
                                    tags,
                                    total_active_sec,
                                    expanded_apps,
                                    on_tag_assigned,
                                );
                            }
                        }
                    });
            });
        });
    });
}

struct AppGroup {
    name: String,
    total_seconds: i64,
    tag_id: Option<i64>,
    sub_items: Vec<(String, i64, Option<i64>)>, // title, duration, tag_id
}

fn build_app_groups(
    activities: &[ActivitySession],
    tags: &[Tag],
    cat_filter: Option<&str>,
    search: &str,
) -> Vec<AppGroup> {
    let mut map: BTreeMap<String, (i64, Option<i64>, BTreeMap<String, (i64, Option<i64>)>)> = BTreeMap::new();
    let search_lower = search.to_lowercase();

    for act in activities {
        if act.is_afk {
            continue;
        }

        if let Some(target_cat) = cat_filter {
            let (act_cat, _) = resolve_tag_info(act.tag_id, tags);
            if !act_cat.eq_ignore_ascii_case(target_cat) {
                continue;
            }
        }

        let entry = map.entry(act.app_name.clone()).or_insert_with(|| (0, act.tag_id, BTreeMap::new()));
        entry.0 += act.duration;

        let title_clean = if let Some(ref d) = act.domain {
            format!("{} • {}", d, act.window_title)
        } else if !act.window_title.is_empty() {
            act.window_title.clone()
        } else {
            "(untitled)".to_string()
        };

        let sub = entry.2.entry(title_clean).or_insert((0, act.tag_id));
        sub.0 += act.duration;
    }

    let mut result = Vec::new();
    for (app_name, (app_secs, app_tag, sub_items_map)) in map {
        let mut sub_items: Vec<(String, i64, Option<i64>)> = Vec::new();
        for (sub_name, (sub_secs, sub_tag)) in sub_items_map {
            if !search_lower.is_empty()
                && !sub_name.to_lowercase().contains(&search_lower)
                && !app_name.to_lowercase().contains(&search_lower)
            {
                continue;
            }
            sub_items.push((sub_name, sub_secs, sub_tag));
        }

        if !search_lower.is_empty() && sub_items.is_empty() && !app_name.to_lowercase().contains(&search_lower) {
            continue;
        }

        sub_items.sort_by(|a, b| b.1.cmp(&a.1));
        result.push(AppGroup {
            name: app_name,
            total_seconds: app_secs,
            tag_id: app_tag,
            sub_items,
        });
    }

    result.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));
    result
}

fn render_app_row(
    ui: &mut Ui,
    app: &AppGroup,
    tags: &[Tag],
    total_active_sec: i64,
    expanded_apps: &mut HashSet<String>,
    _on_tag_assigned: &mut Option<(String, i64)>,
) {
    let theme = Theme::DARK;
    let is_expanded = expanded_apps.contains(&app.name);
    let ratio = (app.total_seconds as f32 / total_active_sec.max(1) as f32).clamp(0.0, 1.0);
    let (_, color_hex) = resolve_tag_info(app.tag_id, tags);
    let tag_color = parse_hex_color(&color_hex);

    ui.horizontal(|ui| {
        // Expand/collapse chevron button
        let chevron = if is_expanded { "▾" } else { "▸" };
        if ui.button(RichText::new(chevron).size(13.0).color(theme.text_muted)).clicked() {
            if is_expanded {
                expanded_apps.remove(&app.name);
            } else {
                expanded_apps.insert(app.name.clone());
            }
        }

        // Color dot
        let (dot, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
        ui.painter().circle_filled(dot.center(), 4.0, tag_color);

        // App name (clickable to toggle expansion)
        let name_btn = ui.add(
            egui::Button::new(RichText::new(&app.name).size(13.0).strong().color(theme.text_primary))
                .frame(false),
        );
        if name_btn.clicked() {
            if is_expanded {
                expanded_apps.remove(&app.name);
            } else {
                expanded_apps.insert(app.name.clone());
            }
        }

        // Sub-item count indicator
        if !app.sub_items.is_empty() {
            ui.label(RichText::new(format!("({} tasks)", app.sub_items.len())).size(11.0).color(theme.text_dim));
        }

        // Right side: Progress Bar & Duration
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let dur_str = format_pt_time(app.total_seconds);
            let pct_str = format!("{:>5.1}%", ratio * 100.0);
            ui.label(RichText::new(format!("{}  {}", pct_str, dur_str)).size(11.5).monospace().color(theme.text_muted));

            ui.add_space(8.0);

            // Progress bar
            let bar_w = 120.0;
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 6.0), egui::Sense::hover());
            ui.painter().rect_filled(bar_rect, Rounding::same(3.0), Color32::from_rgb(32, 36, 45));

            let fill_w = (bar_w * ratio).max(2.0);
            let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
            ui.painter().rect_filled(fill_rect, Rounding::same(3.0), tag_color);
        });
    });

    // Expanded drill-down sub-items
    if is_expanded {
        for (sub_title, sub_secs, _) in &app.sub_items {
            let sub_ratio = (*sub_secs as f32 / app.total_seconds.max(1) as f32).clamp(0.0, 1.0);

            ui.horizontal(|ui| {
                ui.add_space(28.0);
                ui.label(RichText::new("•").size(10.0).color(theme.text_dim));
                ui.label(RichText::new(truncate_sub(sub_title, 60)).size(11.5).color(theme.text_muted));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let sub_dur = format_pt_time(*sub_secs);
                    let sub_pct = format!("{:>4.0}%", sub_ratio * 100.0);
                    ui.label(RichText::new(format!("{}  {}", sub_pct, sub_dur)).size(11.0).monospace().color(theme.text_dim));
                });
            });
        }
        ui.add_space(4.0);
    }
    ui.separator();
}

fn resolve_tag_info(tag_id: Option<i64>, tags: &[Tag]) -> (String, String) {
    if let Some(id) = tag_id {
        if let Some(t) = tags.iter().find(|t| t.id == Some(id)) {
            return (t.name.clone(), t.color.clone());
        }
    }
    ("UNTAGGED".to_string(), "#64748B".to_string())
}

fn truncate_sub(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    }
}
