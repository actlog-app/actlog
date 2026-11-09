use iced::border::Radius;
use iced::widget::slider::{Handle, HandleShape, Rail};
use iced::widget::{button, container, pick_list, slider, text_input};
use iced::{Background, Border, Color, Shadow};

#[derive(Clone, Copy, Debug)]
pub struct Win32Theme;

impl Win32Theme {
    // Standard Windows Classic / Modern Win32 Palette
    pub const DIALOG_BG: Color = Color::from_rgb(240.0 / 255.0, 240.0 / 255.0, 240.0 / 255.0); // #F0F0F0
    pub const WHITE_PANEL: Color = Color::WHITE; // #FFFFFF
    pub const PANEL_BORDER: Color = Color::from_rgb(122.0 / 255.0, 122.0 / 255.0, 122.0 / 255.0); // #7A7A7A
    pub const GROUPBOX_BORDER: Color = Color::from_rgb(208.0 / 255.0, 208.0 / 255.0, 208.0 / 255.0); // #D0D0D0
    pub const BUTTON_FACE: Color = Color::from_rgb(225.0 / 255.0, 225.0 / 255.0, 225.0 / 255.0); // #E1E1E1
    pub const BUTTON_HOVER: Color = Color::from_rgb(229.0 / 255.0, 241.0 / 255.0, 251.0 / 255.0); // #E5F1FB
    pub const BUTTON_BORDER: Color = Color::from_rgb(173.0 / 255.0, 173.0 / 255.0, 173.0 / 255.0); // #ADADAD
    pub const BUTTON_HOVER_BORDER: Color = Color::from_rgb(0.0 / 255.0, 120.0 / 255.0, 215.0 / 255.0);
    pub const TEXT_BLACK: Color = Color::BLACK; // #000000
    pub const TEXT_MUTED: Color = Color::from_rgb(100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0);
    pub const TEXT_DIM: Color = Color::from_rgb(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0);
    pub const SELECTION_BG: Color = Color::from_rgb(205.0 / 255.0, 232.0 / 255.0, 255.0 / 255.0); // #CDE8FF
    pub const SELECTION_BORDER: Color = Color::from_rgb(153.0 / 255.0, 209.0 / 255.0, 255.0 / 255.0); // #99D1FF
    pub const UNTAGGED_GRAY: Color = Color::from_rgb(208.0 / 255.0, 208.0 / 255.0, 208.0 / 255.0); // #D0D0D0
}

pub fn dialog_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(Win32Theme::TEXT_BLACK),
        background: Some(Background::Color(Win32Theme::DIALOG_BG)),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn white_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(Win32Theme::TEXT_BLACK),
        background: Some(Background::Color(Win32Theme::WHITE_PANEL)),
        border: Border {
            color: Win32Theme::PANEL_BORDER,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn groupbox_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(Win32Theme::TEXT_BLACK),
        background: Some(Background::Color(Win32Theme::DIALOG_BG)),
        border: Border {
            color: Win32Theme::GROUPBOX_BORDER,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn win32_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let (bg, border_color) = match status {
        button::Status::Hovered => (Win32Theme::BUTTON_HOVER, Win32Theme::BUTTON_HOVER_BORDER),
        button::Status::Pressed => (Win32Theme::SELECTION_BG, Win32Theme::BUTTON_HOVER_BORDER),
        button::Status::Disabled => (Win32Theme::DIALOG_BG, Win32Theme::GROUPBOX_BORDER),
        _ => (Win32Theme::BUTTON_FACE, Win32Theme::BUTTON_BORDER),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Win32Theme::TEXT_BLACK,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::from(2.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn win32_text_input_style(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Win32Theme::WHITE_PANEL),
        border: Border {
            color: Win32Theme::PANEL_BORDER,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        icon: Win32Theme::TEXT_MUTED,
        placeholder: Win32Theme::TEXT_DIM,
        value: Win32Theme::TEXT_BLACK,
        selection: Win32Theme::SELECTION_BG,
    }
}

pub fn win32_pick_list_style(_theme: &iced::Theme, _status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: Win32Theme::TEXT_BLACK,
        placeholder_color: Win32Theme::TEXT_DIM,
        handle_color: Win32Theme::TEXT_BLACK,
        background: Background::Color(Win32Theme::WHITE_PANEL),
        border: Border {
            color: Win32Theme::PANEL_BORDER,
            width: 1.0,
            radius: Radius::from(0.0),
        },
    }
}

pub fn win32_slider_style(_theme: &iced::Theme, status: slider::Status) -> slider::Style {
    let handle_bg = match status {
        slider::Status::Hovered | slider::Status::Dragged => Win32Theme::BUTTON_HOVER,
        _ => Win32Theme::BUTTON_FACE,
    };

    slider::Style {
        rail: Rail {
            backgrounds: (
                Background::Color(Win32Theme::GROUPBOX_BORDER),
                Background::Color(Win32Theme::GROUPBOX_BORDER),
            ),
            width: 3.0,
            border: Border::default(),
        },
        handle: Handle {
            shape: HandleShape::Rectangle {
                width: 8,
                border_radius: Radius::from(1.0).into(),
            },
            background: Background::Color(handle_bg),
            border_width: 1.0,
            border_color: Win32Theme::PANEL_BORDER,
        },
    }
}
