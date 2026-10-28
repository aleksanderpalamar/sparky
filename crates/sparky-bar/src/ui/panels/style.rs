use iced::{
    Border,
    widget::{container, slider},
};
use sparky_theme::Skin;

pub fn surface(skin: Skin) -> container::Style {
    container::Style {
        background: Some(
            iced::Color {
                a: 1.0,
                ..skin.background
            }
            .into(),
        ),
        text_color: Some(skin.foreground),
        border: Border {
            color: skin.accent,
            width: 2.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn volume(skin: Skin) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
    move |_, _| slider::Style {
        rail: slider::Rail {
            backgrounds: (skin.accent.into(), skin.selection.into()),
            width: 4.0,
            border: Border::default(),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Rectangle {
                width: 8,
                border_radius: 0.0.into(),
            },
            background: skin.foreground.into(),
            border_width: 0.0,
            border_color: skin.accent,
        },
    }
}
