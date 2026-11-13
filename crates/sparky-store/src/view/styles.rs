use iced::widget::{button, container};
use iced::{Background, Border};
use sparky_theme::Skin;

pub fn surface(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.lighter_background)),
        border: Border {
            color: skin.fill(0.15),
            width: 1.0,
            ..Border::default()
        },
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}

pub fn feature(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.selection)),
        border: Border {
            color: skin.accent,
            width: 1.0,
            ..Border::default()
        },
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}

pub fn tile(skin: Skin, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => skin.selection,
        _ => skin.lighter_background,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: skin.foreground,
        border: Border {
            color: skin.fill(0.15),
            width: 1.0,
            ..Border::default()
        },
        ..button::Style::default()
    }
}

pub fn tab(skin: Skin, selected: bool, status: button::Status) -> button::Style {
    let mut style = tile(skin, status);
    if selected {
        style.background = Some(Background::Color(skin.selection));
        style.border.color = skin.accent;
    }
    style
}

pub fn topic(skin: Skin, index: usize, status: button::Status) -> button::Style {
    let mut style = tile(skin, status);
    if !matches!(status, button::Status::Hovered | button::Status::Pressed) {
        style.background = Some(Background::Color(match index % 3 {
            0 => skin.accent_fill(0.22),
            1 => iced::Color {
                a: 0.2,
                ..skin.cyan
            },
            _ => skin.selection,
        }));
    }
    style
}

pub fn action(skin: Skin, status: button::Status) -> button::Style {
    let mut style = tile(skin, status);
    style.background = Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => skin.selection,
        _ => skin.accent_fill(0.3),
    }));
    style.border.color = skin.accent;
    style
}

#[cfg(test)]
mod tests {
    use super::{feature, surface, tab};
    use iced::widget::button;
    use sparky_theme::Skin;

    #[test]
    fn superficies_preservam_cantos_retos_e_cores_do_tema() {
        let skin = Skin::default();
        let feature = feature(skin);
        let panel = surface(skin);
        let selected = tab(skin, true, button::Status::Active);
        assert_eq!(feature.border.radius, 0.0.into());
        assert_eq!(panel.border.radius, 0.0.into());
        assert_eq!(selected.border.radius, 0.0.into());
        assert_eq!(feature.border.color, skin.accent);
        assert_eq!(selected.border.color, skin.accent);
    }
}
