use iced::widget::{button, checkbox, container};
use iced::{Background, Border, Color};
use sparky_theme::Skin;

use crate::domain::presence::Presence;

const OUTLINE: f32 = 1.0;
const DIM: f32 = 0.55;

pub fn tray(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(opaque(skin.lighter_background))),
        border: outline(skin.accent_fill(0.35)),
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}

pub fn slot(skin: Skin, presence: Presence, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let background = match (presence, hovered) {
        (Presence::Focused, _) => skin.accent_fill(0.55),
        (_, true) => skin.accent_fill(0.3),
        (_, false) => skin.fill(0.08),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: skin.accent,
        ..button::Style::default()
    }
}

pub fn primary(skin: Skin, status: button::Status) -> button::Style {
    let (background, text_color) = match status {
        button::Status::Disabled => (skin.fill(0.06), skin.muted),
        button::Status::Hovered | button::Status::Pressed => (skin.accent, skin.background),
        button::Status::Active => (skin.selection, skin.bright_foreground),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: outline(skin.accent),
        ..button::Style::default()
    }
}

pub fn mark(skin: Skin, presence: Presence) -> Option<Color> {
    match presence {
        Presence::Closed => None,
        Presence::Running => Some(skin.muted),
        Presence::Focused => Some(skin.accent),
    }
}

pub fn dot(color: Color, size: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: (size / 2.0).into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn divider(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.fill(0.15))),
        ..container::Style::default()
    }
}

pub fn backdrop(_: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, DIM))),
        ..container::Style::default()
    }
}

pub fn dialog(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(opaque(skin.lighter_background))),
        border: outline(skin.accent),
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}

pub fn card(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.fill(0.04))),
        ..container::Style::default()
    }
}

pub fn check(skin: Skin, status: checkbox::Status) -> checkbox::Style {
    let (checked, enabled) = match status {
        checkbox::Status::Active { is_checked } | checkbox::Status::Hovered { is_checked } => {
            (is_checked, true)
        }
        checkbox::Status::Disabled { is_checked } => (is_checked, false),
    };
    let tone = match enabled {
        true => skin.accent,
        false => skin.muted,
    };
    checkbox::Style {
        background: Background::Color(match checked {
            true => tone,
            false => Color::TRANSPARENT,
        }),
        icon_color: skin.background,
        border: outline(tone),
        text_color: Some(skin.foreground),
    }
}

fn outline(color: Color) -> Border {
    Border {
        color,
        width: OUTLINE,
        ..Border::default()
    }
}

fn opaque(color: Color) -> Color {
    Color { a: 1.0, ..color }
}
