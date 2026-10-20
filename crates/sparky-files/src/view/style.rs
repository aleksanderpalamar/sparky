use iced::widget::{button, container};
use iced::{Background, Border, Color};
use sparky_theme::Skin;

pub const APPLICATION_ID: &str = "sparky-files";
pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
pub const SIDEBAR_WIDTH: f32 = 210.0;
pub const HEADER_HEIGHT: f32 = 44.0;
pub const ROW_HEIGHT: f32 = 28.0;
pub const STATUS_HEIGHT: f32 = 36.0;
pub const TEXT_SIZE: f32 = 14.0;
pub const SMALL_TEXT: f32 = 12.0;
pub const PADDING: f32 = 12.0;
pub const SPACING: f32 = 8.0;
pub const SIZE_WIDTH: f32 = 90.0;
pub const DATE_WIDTH: f32 = 140.0;
const CHROME: f32 = HEADER_HEIGHT + ROW_HEIGHT + STATUS_HEIGHT;
const MARKER: f32 = 1.0;
const RADIUS: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    Plain,
    Selected,
}

pub fn rows_for(height: f32) -> usize {
    ((height - CHROME) / ROW_HEIGHT).floor().max(1.0) as usize
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
                width: MARKER,
                radius: RADIUS.into(),
            },
            ..container::Style::default()
        },
    }
}

pub fn place(skin: Skin, highlight: Highlight, status: button::Status) -> button::Style {
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
    use super::{CHROME, ROW_HEIGHT, rows_for};

    #[test]
    fn as_linhas_cabem_no_que_sobra_da_janela() {
        assert_eq!(rows_for(CHROME + ROW_HEIGHT * 10.0 + 5.0), 10);
    }

    #[test]
    fn janela_minuscula_ainda_mostra_uma_linha() {
        assert_eq!(rows_for(10.0), 1);
    }
}
