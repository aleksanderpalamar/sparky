use iced::widget::container;
use iced::{Background, Border, Color};
use sparky_theme::Skin;

pub fn floating(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: skin.background.a,
            ..skin.raised
        })),
        text_color: Some(skin.foreground),
        border: Border {
            color: skin.accent,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
