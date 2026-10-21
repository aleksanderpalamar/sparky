use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};
use sparky_theme::Skin;

pub const TEXT_SIZE: f32 = 14.0;
pub const SMALL_TEXT: f32 = 12.0;
pub const TITLE_SIZE: f32 = 16.0;
pub const PADDING: f32 = 12.0;
pub const SPACING: f32 = 8.0;
pub const SIDEBAR_WIDTH: f32 = 210.0;
pub const HEADER_HEIGHT: f32 = 44.0;
pub const STATUS_HEIGHT: f32 = 36.0;
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
                ..Border::default()
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
        ..button::Style::default()
    }
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let standard = text_input::default(theme, status);
    text_input::Style {
        border: Border { radius: 0.0.into(), ..standard.border },
        ..standard
    }
}

#[cfg(test)]
mod tests {
    use iced::border::Radius;
    use iced::widget::{button, text_input};
    use sparky_theme::Skin;

    use super::{Highlight, entry, input, row};

    #[test]
    fn o_destaque_vem_da_selecao() {
        assert_eq!(Highlight::when(true), Highlight::Selected);
        assert_eq!(Highlight::when(false), Highlight::Plain);
    }

    #[test]
    fn a_selecao_e_os_botoes_tem_cantos_retos_como_as_janelas() {
        let skin = Skin::default();
        let reto = Radius::from(0.0);
        assert_eq!(row(skin, Highlight::Selected).border.radius, reto);
        assert_eq!(entry(skin, Highlight::Selected, button::Status::Active).border.radius, reto);
        assert_eq!(entry(skin, Highlight::Plain, button::Status::Hovered).border.radius, reto);
    }

    #[test]
    fn a_caixa_de_texto_tem_cantos_retos() {
        let theme = Skin::default().theme();
        assert_eq!(input(&theme, text_input::Status::Active).border.radius, Radius::from(0.0));
        assert_eq!(input(&theme, text_input::Status::Focused { is_hovered: false }).border.radius, Radius::from(0.0));
    }
}
