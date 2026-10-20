use iced::Theme;
use iced::theme::Palette;
use sparky_theme::Skin;

pub fn theme(skin: &Skin) -> Theme {
    Theme::custom(
        "sparky".to_owned(),
        Palette {
            background: skin.background,
            text: skin.foreground,
            primary: skin.accent,
            success: skin.cyan,
            warning: skin.accent,
            danger: skin.urgent,
        },
    )
}
