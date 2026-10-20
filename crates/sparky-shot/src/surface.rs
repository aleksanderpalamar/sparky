use iced::theme::Style;
use iced::{Color, Theme};

pub fn transparent<State>(_state: &State, theme: &Theme) -> Style {
    Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    }
}
