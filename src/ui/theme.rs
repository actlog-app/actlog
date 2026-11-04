use egui::{Color32, Margin, Rounding, Stroke, Style, Vec2, Visuals};

pub struct Theme {
    pub bg_dark: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
}

impl Theme {
    pub const DARK: Self = Self {
        bg_dark: Color32::from_rgb(13, 15, 18),        // Deep window backdrop
        surface: Color32::from_rgb(20, 23, 29),        // Cards, panels, input fields
        surface_hover: Color32::from_rgb(28, 32, 40),  // Interactive hover
        surface_active: Color32::from_rgb(36, 41, 52), // Pressed / active
        border: Color32::from_rgb(38, 43, 54),         // Clean subtle border
        border_subtle: Color32::from_rgb(28, 32, 40),  // Inner dividers
        text_primary: Color32::from_rgb(241, 245, 249), // Crisp text
        text_muted: Color32::from_rgb(148, 163, 184),  // Slate-400
        text_dim: Color32::from_rgb(100, 116, 139),    // Slate-500
        accent: Color32::from_rgb(99, 102, 241),       // Indigo-500
        accent_hover: Color32::from_rgb(129, 140, 248),// Indigo-400
    };
}

pub fn configure_impeccable_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    let theme = Theme::DARK;

    visuals.dark_mode = true;
    visuals.override_text_color = Some(theme.text_primary);
    visuals.window_fill = theme.bg_dark;
    visuals.panel_fill = theme.bg_dark;

    // Widget states
    visuals.widgets.noninteractive.bg_fill = theme.surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_muted);
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

    visuals.widgets.inactive.bg_fill = theme.surface;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = theme.surface_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    visuals.widgets.active.bg_fill = theme.surface_active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.accent_hover);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    visuals.widgets.open.bg_fill = theme.surface_hover;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.accent);

    // Text selection & focus
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(99, 102, 241, 75);
    visuals.selection.stroke = Stroke::new(1.0, theme.accent);

    // Windows & Separators
    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0, theme.border);

    let mut style = Style::default();
    style.visuals = visuals;
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.window_margin = Margin::same(12.0);

    ctx.set_style(style);
}

