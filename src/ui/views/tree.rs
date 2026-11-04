use crate::db::models::{ActivitySession, Tag};
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

        // Group by app_name -> window_title
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
                "(no title)".to_string()
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

            // Sort children by duration desc
            children.sort_by(|a, b| b.total_seconds.cmp(&a.total_seconds));

            roots.push(ActivityNode {
                name: app_name,
                total_seconds: app_secs,
                tag_id: app_tag,
                children,
                is_expanded: expand_level >= 2,
            });
        }

        // Sort roots by duration desc (Procrastitracker ranking)
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
    let row_height = 20.0;
    let max_seconds = total_db_seconds.max(1) as f32;

    for node in nodes.iter_mut() {
        render_tree_node(
            ui,
            node,
            tags,
            max_seconds,
            0,
            row_height,
            selected_node_name,
        );
    }
}

fn render_tree_node(
    ui: &mut Ui,
    node: &mut ActivityNode,
    tags: &[Tag],
    max_seconds: f32,
    depth: usize,
    row_height: f32,
    selected_node_name: &mut Option<String>,
) {
    let has_children = !node.children.is_empty();
    let indent = (depth as f32) * 16.0;

    ui.horizontal(|ui| {
        ui.add_space(indent);

        // Expand/collapse button
        if has_children {
            let symbol = if node.is_expanded { "[-] " } else { "[+] " };
            if ui.button(egui::RichText::new(symbol).monospace().size(11.0)).clicked() {
                node.is_expanded = !node.is_expanded;
            }
        } else {
            ui.add_space(18.0);
        }

        // Color tag swatch
        let tag_color = resolve_tag_color(node.tag_id, tags);
        let (swatch_rect, _) =
            ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
        ui.painter().rect_filled(swatch_rect, Rounding::ZERO, tag_color);
        ui.painter().rect_stroke(
            swatch_rect,
            Rounding::ZERO,
            Stroke::new(1.0, Color32::from_rgb(100, 100, 100)),
        );

        // Name (selectable)
        let is_selected = selected_node_name.as_deref() == Some(&node.name);
        let name_label = egui::SelectableLabel::new(
            is_selected,
            egui::RichText::new(&node.name).size(12.0).color(Color32::BLACK),
        );
        if ui.add(name_label).clicked() {
            *selected_node_name = Some(node.name.clone());
        }

        // Stacked colored bar graph (Procrastitracker style)
        let bar_width = 160.0;
        let pct = (node.total_seconds as f32 / max_seconds).clamp(0.0, 1.0);
        let fill_w = (bar_width * pct).max(2.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Percentage & Time label
            let time_str = format_pt_time(node.total_seconds);
            let pct_str = format!("{:>5.1}%", pct * 100.0);
            ui.label(
                egui::RichText::new(format!("{}  {}", pct_str, time_str))
                    .monospace()
                    .size(11.0)
                    .color(Color32::from_rgb(60, 60, 60)),
            );

            // Bar box
            let (bar_rect, _) =
                ui.allocate_exact_size(Vec2::new(bar_width, 12.0), egui::Sense::hover());
            ui.painter().rect_filled(bar_rect, Rounding::ZERO, Color32::from_rgb(230, 230, 230));
            ui.painter().rect_stroke(
                bar_rect,
                Rounding::ZERO,
                Stroke::new(1.0, Color32::from_rgb(180, 180, 180)),
            );

            let filled_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
            ui.painter().rect_filled(filled_rect, Rounding::ZERO, tag_color);
        });
    });

    if node.is_expanded && has_children {
        for child in node.children.iter_mut() {
            render_tree_node(
                ui,
                child,
                tags,
                max_seconds,
                depth + 1,
                row_height,
                selected_node_name,
            );
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
    Color32::from_rgb(208, 208, 208) // UNTAGGED default
}
