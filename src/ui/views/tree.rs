use crate::db::models::{ActivitySession, Tag};
use crate::ui::theme::Theme;
use crate::ui::views::timeline::parse_hex_color;
use egui::{Color32, Rect, Rounding, Stroke, Ui, Vec2};

#[derive(Debug, Clone)]
pub struct ActivityNode {
    pub name: String,
    pub total_seconds: i64,
    pub tag_id: Option<i64>,
    pub children: Vec<ActivityNode>,
    pub is_expanded: bool,
}

impl ActivityNode {
    pub fn build_tree(
        activities: &[ActivitySession],
        expand_level: u32,
        min_seconds: i64,
        substring_filter: &str,
        tag_filter: Option<i64>,
    ) -> Vec<ActivityNode> {
        use std::collections::BTreeMap;

        let mut app_map: BTreeMap<String, (i64, Option<i64>, BTreeMap<String, (i64, Option<i64>)>)> =
            BTreeMap::new();

        for act in activities {
            if act.is_afk {
                continue;
            }
            if let Some(target_tag) = tag_filter {
                if act.tag_id != Some(target_tag) {
                    continue;
                }
            }

            let entry = app_map
                .entry(act.app_name.clone())
                .or_insert_with(|| (0, act.tag_id, BTreeMap::new()));
            entry.0 += act.duration;

            let sub_title = if let Some(ref domain) = act.domain {
                format!("{} - {}", domain, act.window_title)
            } else if !act.window_title.is_empty() {
                act.window_title.clone()
            } else {
                "(untitled)".to_string()
            };

            let sub_entry = entry.2.entry(sub_title).or_insert((0, act.tag_id));
            sub_entry.0 += act.duration;
        }

        let filter_lower = substring_filter.to_lowercase();
        let mut roots: Vec<ActivityNode> = Vec::new();

        for (app_name, (app_secs, app_tag, sub_items)) in app_map {
            if app_secs < min_seconds {
                continue;
            }

            let mut children: Vec<ActivityNode> = Vec::new();
            for (sub_name, (sub_secs, sub_tag)) in sub_items {
                if sub_secs < min_seconds {
                    continue;
                }
                if !filter_lower.is_empty()
                    && !sub_name.to_lowercase().contains(&filter_lower)
                    && !app_name.to_lowercase().contains(&filter_lower)
                {
                    continue;
                }

                children.push(ActivityNode {
                    name: sub_name,
                    total_seconds: sub_secs,
                    tag_id: sub_tag,
                    children: Vec::new(),
                    is_expanded: false,
                });
            }

            if !filter_lower.is_empty()
                && children.is_empty()
                && !app_name.to_lowercase().contains(&filter_lower)
            {
                continue;
            }

            children.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

            roots.push(ActivityNode {
                name: app_name,
                total_seconds: app_secs,
                tag_id: app_tag,
                children,
                is_expanded: expand_level >= 2,
            });
        }

        roots.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));
        roots
    }
}

pub fn render_procrastitracker_tree(
    ui: &mut Ui,
    nodes: &mut [ActivityNode],
    tags: &[Tag],
    total_db_seconds: i64,
    selected_node_name: &mut Option<String>,
) {
    let max_seconds = total_db_seconds.max(1) as f32;

    for node in nodes.iter_mut() {
        render_tree_node(ui, node, tags, max_seconds, 0, selected_node_name);
    }
}

fn render_tree_node(
    ui: &mut Ui,
    node: &mut ActivityNode,
    tags: &[Tag],
    max_seconds: f32,
    depth: usize,
    selected_node_name: &mut Option<String>,
) {
    let has_children = !node.children.is_empty();
    let indent = (depth as f32) * 18.0;
    let theme = Theme::DARK;

    let is_selected = selected_node_name.as_deref() == Some(&node.name);

    let row_response = ui.horizontal(|ui| {
        ui.add_space(indent);

        // Modern Chevron Toggle
        if has_children {
            let chevron = if node.is_expanded { "▾" } else { "▸" };
            let chevron_btn = ui.add(
                egui::Button::new(egui::RichText::new(chevron).size(12.0).color(theme.text_muted))
                    .frame(false)
                    .min_size(Vec2::new(14.0, 14.0)),
            );
            if chevron_btn.clicked() {
                node.is_expanded = !node.is_expanded;
            }
        } else {
            ui.add_space(18.0);
        }

        // Color Chip Pill
        let tag_color = resolve_tag_color(node.tag_id, tags);
        let (chip_rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
        ui.painter().rect_filled(chip_rect, Rounding::same(3.0), tag_color);
        ui.painter().rect_stroke(
            chip_rect,
            Rounding::same(3.0),
            Stroke::new(1.0, Color32::from_black_alpha(80)),
        );

        ui.add_space(2.0);

        // Name
        let text_color = if is_selected {
            theme.accent_hover
        } else if depth == 0 {
            theme.text_primary
        } else {
            theme.text_muted
        };

        let label_text = egui::RichText::new(&node.name)
            .size(12.5)
            .strong()
            .color(text_color);

        if ui.add(egui::SelectableLabel::new(is_selected, label_text)).clicked() {
            *selected_node_name = Some(node.name.clone());
        }

        // Right-aligned Progress Bar & Duration
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let time_str = format_pt_time(node.total_seconds);
            let pct = (node.total_seconds as f32 / max_seconds).clamp(0.0, 1.0);
            let pct_str = format!("{:>5.1}%", pct * 100.0);

            ui.label(
                egui::RichText::new(format!("{}  {}", pct_str, time_str))
                    .monospace()
                    .size(11.5)
                    .color(theme.text_muted),
            );

            ui.add_space(8.0);

            // Sleek Rounded Bar
            let bar_width = 150.0;
            let (bar_rect, _) =
                ui.allocate_exact_size(Vec2::new(bar_width, 8.0), egui::Sense::hover());

            // Track background
            ui.painter().rect_filled(
                bar_rect,
                Rounding::same(4.0),
                Color32::from_rgb(32, 36, 45),
            );

            // Filled progress
            let fill_w = (bar_width * pct).max(3.0);
            let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
            ui.painter().rect_filled(fill_rect, Rounding::same(4.0), tag_color);
        });
    });

    if row_response.response.hovered() && !is_selected {
        ui.painter().rect_stroke(
            row_response.response.rect,
            Rounding::same(4.0),
            Stroke::new(1.0, theme.border_subtle),
        );
    }

    if node.is_expanded && has_children {
        for child in node.children.iter_mut() {
            render_tree_node(ui, child, tags, max_seconds, depth + 1, selected_node_name);
        }
    }
}

pub fn format_pt_time(seconds: i64) -> String {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

fn resolve_tag_color(tag_id: Option<i64>, tags: &[Tag]) -> Color32 {
    if let Some(id) = tag_id {
        if let Some(t) = tags.iter().find(|t| t.id == Some(id)) {
            return parse_hex_color(&t.color);
        }
    }
    Color32::from_rgb(100, 116, 139) // Slate-500 UNTAGGED default
}
