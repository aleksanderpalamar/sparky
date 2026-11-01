use iced::widget::{button, container, overlay::menu, slider, text_input, toggler};
use iced::{Border, Color, Theme};
use sparky_theme::Skin;

use super::tokens::{RADIUS, alpha};

pub fn border(color: Color, width: f32) -> Border {
    Border {
        color,
        width,
        radius: RADIUS.into(),
    }
}

pub fn surface(color: Color) -> container::Style {
    container::Style {
        background: Some(color.into()),
        ..container::Style::default()
    }
}

pub fn selectable(skin: Skin, selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (fill, edge) = match (selected, hovered) {
            (true, _) => (
                skin.fill(alpha::SELECTED_FILL),
                border(skin.foreground, 1.0),
            ),
            (false, true) => (
                skin.fill(alpha::HOVER_FILL),
                border(skin.fill(alpha::HOVER_BORDER), 1.0),
            ),
            (false, false) => (Color::TRANSPARENT, border(Color::TRANSPARENT, 0.0)),
        };
        button::Style {
            background: Some(fill.into()),
            text_color: skin.foreground,
            border: edge,
            ..button::Style::default()
        }
    }
}

pub fn frame(skin: Skin, resting: f32) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let (fill, edge) = match hovered {
            true => (alpha::HOVER_FILL, alpha::NORMAL_BORDER),
            false => (alpha::NORMAL_FILL, resting),
        };
        button::Style {
            background: Some(skin.fill(fill).into()),
            text_color: skin.foreground,
            border: border(skin.fill(edge), 1.0),
            ..button::Style::default()
        }
    }
}

pub fn field(skin: Skin) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let highlighted = matches!(
            status,
            text_input::Status::Hovered | text_input::Status::Focused { .. }
        );
        let (fill, edge) = match highlighted {
            true => (alpha::HOVER_FILL, alpha::NORMAL_BORDER),
            false => (alpha::NORMAL_FILL, alpha::HOVER_BORDER),
        };
        text_input::Style {
            background: skin.fill(fill).into(),
            border: border(skin.fill(edge), 1.0),
            icon: skin.dark_foreground,
            placeholder: skin.dark_foreground,
            value: skin.foreground,
            selection: skin.accent_fill(alpha::SELECTED_FILL),
        }
    }
}

pub fn menu(skin: Skin) -> impl Fn(&Theme) -> menu::Style {
    move |_| menu::Style {
        background: skin.lighter_background.into(),
        border: border(skin.fill(alpha::NORMAL_BORDER), 1.0),
        text_color: skin.foreground,
        selected_text_color: skin.background,
        selected_background: skin.accent.into(),
        shadow: iced::Shadow::default(),
    }
}

pub fn rail(skin: Skin) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_, _| slider::Style {
        rail: slider::Rail {
            backgrounds: (skin.accent.into(), skin.fill(alpha::SELECTED_FILL).into()),
            width: 4.0,
            border: border(Color::TRANSPARENT, 0.0),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Rectangle {
                width: 6,
                border_radius: RADIUS.into(),
            },
            background: skin.foreground.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub fn switch(skin: Skin) -> impl Fn(&Theme, toggler::Status) -> toggler::Style {
    move |_, status| {
        let (checked, hovered) = match status {
            toggler::Status::Active { is_toggled } | toggler::Status::Disabled { is_toggled } => {
                (is_toggled, false)
            }
            toggler::Status::Hovered { is_toggled } => (is_toggled, true),
        };
        let track = match (checked, hovered) {
            (true, _) => alpha::SELECTED_FILL,
            (false, true) => alpha::HOVER_FILL,
            (false, false) => alpha::NORMAL_FILL,
        };
        let (edge, knob) = match checked {
            true => (skin.foreground, skin.foreground),
            false => (skin.fill(alpha::NORMAL_BORDER), skin.dark_foreground),
        };
        toggler::Style {
            background: skin.fill(track).into(),
            background_border_width: 1.0,
            background_border_color: edge,
            foreground: knob.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            border_radius: Some(RADIUS.into()),
            padding_ratio: 0.14,
            text_color: None,
        }
    }
}
