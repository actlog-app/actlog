use egui::{Color32, Margin, Rounding, Stroke, Style, Vec2, Visuals};

#[derive(Clone, Copy)]
pub struct Theme {
    pub dialog_bg: Color32,       // RGB(240, 240, 240) - COLOR_3DFACE
    pub control_bg: Color32,      // RGB(255, 255, 255) - Pure white client areas
    pub control_border: Color32,  // RGB(122, 122, 122) - Sunken client edge
    pub groupbox_border: Color32, // RGB(208, 208, 208) - Etched groupbox line
    pub text_primary: Color32,    // RGB(0, 0, 0)       - Crisp black text
    pub text_muted: Color32,      // RGB(100, 100, 100) - Muted secondary text
    pub text_dim: Color32,        // RGB(128, 128, 128) - Dim text
    pub selection_bg: Color32,    // RGB(0, 120, 215)   - Windows selection blue
    pub selection_fg: Color32,    // RGB(255, 255, 255) - White selected text
    pub button_face: Color32,     // RGB(225, 225, 225) - Pushbutton face
    pub button_border: Color32,   // RGB(173, 173, 173) - Pushbutton border
    pub button_hover: Color32,    // RGB(229, 241, 251) - Pushbutton hover
    pub tab_active: Color32,      // RGB(255, 255, 255) - Active tab surface
    pub tab_inactive: Color32,    // RGB(230, 230, 230) - Inactive tab surface
    pub accent: Color32,          // RGB(0, 120, 215)
    pub accent_hover: Color32,    // RGB(24, 144, 255)
    pub border: Color32,          // RGB(122, 122, 122)
    pub border_subtle: Color32,   // RGB(208, 208, 208)
    pub surface: Color32,         // RGB(240, 240, 240)
}

impl Theme {
    pub const WIN32: Self = Self {
        dialog_bg: Color32::from_rgb(240, 240, 240),
        control_bg: Color32::from_rgb(255, 255, 255),
        control_border: Color32::from_rgb(122, 122, 122),
        groupbox_border: Color32::from_rgb(208, 208, 208),
        text_primary: Color32::from_rgb(0, 0, 0),
        text_muted: Color32::from_rgb(100, 100, 100),
        text_dim: Color32::from_rgb(128, 128, 128),
        selection_bg: Color32::from_rgb(0, 120, 215),
        selection_fg: Color32::from_rgb(255, 255, 255),
        button_face: Color32::from_rgb(225, 225, 225),
        button_border: Color32::from_rgb(173, 173, 173),
        button_hover: Color32::from_rgb(229, 241, 251),
        tab_active: Color32::from_rgb(255, 255, 255),
        tab_inactive: Color32::from_rgb(230, 230, 230),
        accent: Color32::from_rgb(0, 120, 215),
        accent_hover: Color32::from_rgb(24, 144, 255),
        border: Color32::from_rgb(122, 122, 122),
        border_subtle: Color32::from_rgb(208, 208, 208),
        surface: Color32::from_rgb(240, 240, 240),
    };

    // Alias for compatibility
    pub const DARK: Self = Self::WIN32;
}

pub fn configure_win32_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    let theme = Theme::WIN32;

    // Load Windows system fonts (Segoe UI for native Windows look, Nirmala for Indic/Bengali, MS YaHei for CJK)
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
        fonts.font_data.insert(
            "segoeui".to_owned(),
            egui::FontData::from_owned(bytes),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoeui".to_owned());
    }
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\Nirmala.ttc") {
        fonts.font_data.insert(
            "nirmala".to_owned(),
            egui::FontData::from_owned(bytes),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("nirmala".to_owned());
    }
    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert(
            "msyh".to_owned(),
            egui::FontData::from_owned(bytes),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("msyh".to_owned());
    }
    ctx.set_fonts(fonts);

    visuals.dark_mode = false;
    visuals.override_text_color = Some(theme.text_primary);
    visuals.window_fill = theme.dialog_bg;
    visuals.panel_fill = theme.dialog_bg;

    // Noninteractive widgets (frames, panels)
    visuals.widgets.noninteractive.bg_fill = theme.dialog_bg;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.control_border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.noninteractive.rounding = Rounding::ZERO;

    // Inactive widgets (buttons, inputs)
    visuals.widgets.inactive.bg_fill = theme.button_face;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.button_border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.inactive.rounding = Rounding::same(1.0);

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = theme.button_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.hovered.rounding = Rounding::same(1.0);

    // Active/pressed widgets
    visuals.widgets.active.bg_fill = Color32::from_rgb(204, 228, 247);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.text_primary);
    visuals.widgets.active.rounding = Rounding::same(1.0);

    visuals.selection.bg_fill = theme.selection_bg;
    visuals.selection.stroke = Stroke::new(1.0, theme.selection_bg);

    ctx.set_visuals(visuals);

    let mut style: Style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(4.0, 4.0);
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    style.spacing.window_margin = Margin::same(6.0);

    // Standard Windows 9pt Segoe UI font size scale for crisp readability
    style.text_styles = [
        (egui::TextStyle::Small, egui::FontId::proportional(10.5)),
        (egui::TextStyle::Body, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Heading, egui::FontId::proportional(14.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(12.0)),
    ]
    .into();

    ctx.set_style(style);
}
