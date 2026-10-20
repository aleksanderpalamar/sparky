use iced::widget::{button, container};
use iced::{Background, Border, Color};
use sparky_theme::Skin;

pub const TEXT_SIZE: f32 = 14.0;
pub const SMALL_TEXT: f32 = 12.0;
pub const TITLE_SIZE: f32 = 16.0;
pub const PADDING: f32 = 12.0;
pub const SPACING: f32 = 8.0;
pub const SIDEBAR_WIDTH: f32 = 210.0;
pub const HEADER_HEIGHT: f32 = 44.0;
pub const STATUS_HEIGHT: f32 = 36.0;
const RADIUS: f32 = 4.0;
const OUTLINE: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    Plain,
    Selected,
}

impl Highlight {
    pub fn when(selected: bool) -> Self {
        match selected {
            true => Highlight::Selected,
            false => Highlight::Plain,
        }
    }
}

pub fn window(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.background)),
        text_color: Some(skin.foreground),
        ..container::Style::default()
    }
}

pub fn panel(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.lighter_background)),
        ..window(skin)
    }
}

pub fn row(skin: Skin, highlight: Highlight) -> container::Style {
    match highlight {
        Highlight::Plain => container::Style::default(),
        Highlight::Selected => container::Style {
            background: Some(Background::Color(skin.selection)),
            border: Border {
                color: skin.accent,
                width: OUTLINE,
                radius: RADIUS.into(),
            },
            ..container::Style::default()
        },
    }
}

pub fn entry(skin: Skin, highlight: Highlight, status: button::Status) -> button::Style {
    let background = match (highlight, status) {
        (Highlight::Selected, _) => skin.selection,
        (Highlight::Plain, button::Status::Hovered | button::Status::Pressed) => skin.background,
        (Highlight::Plain, _) => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: skin.foreground,
        border: Border::default().rounded(RADIUS),
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::Highlight;

    #[test]
    fn o_destaque_vem_da_selecao() {
        assert_eq!(Highlight::when(true), Highlight::Selected);
        assert_eq!(Highlight::when(false), Highlight::Plain);
    }
}
