use iced::font::{Family, Weight};
use iced::widget::{button, svg, text_input};
use iced::{Background, Border, Color, Font};
use sparky_theme::Skin;

pub const REGULAR: Font = Font {
    family: Family::Name("Noto Sans"),
    ..Font::DEFAULT
};
pub const BOLD: Font = Font {
    weight: Weight::Bold,
    ..REGULAR
};
const FILL: f32 = 0.2;
const DISABLED: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colors {
    pub clock: Color,
    pub field_text: Color,
    pub icon: Color,
    pub fill: Color,
    pub button_text: Color,
    pub highlight: Color,
    pub warning: Color,
}

impl Colors {
    pub fn new(skin: Skin) -> Self {
        Self {
            clock: skin.accent,
            field_text: skin.accent,
            icon: skin.bright_foreground,
            fill: Color {
                a: FILL,
                ..skin.background
            },
            button_text: skin.bright_foreground,
            highlight: skin.lighter_background,
            warning: skin.urgent,
        }
    }

    pub fn login_text(&self, ready: bool) -> Color {
        match ready {
            true => self.button_text,
            false => Color {
                a: DISABLED,
                ..self.button_text
            },
        }
    }
}

pub fn frame(colors: Colors, focused: bool) -> Border {
    let (width, color) = match focused {
        true => (2.0, colors.highlight),
        false => (1.0, Color::TRANSPARENT),
    };
    Border {
        color,
        width,
        ..Border::default()
    }
}

pub fn field(colors: Colors) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(colors.fill),
            border: frame(colors, focused),
            icon: colors.icon,
            placeholder: colors.icon,
            value: colors.field_text,
            selection: Color {
                a: 0.4,
                ..colors.clock
            },
        }
    }
}

pub fn login(
    colors: Colors,
    focused: bool,
    ready: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let fill = match hovered && ready {
            true => Color {
                a: 1.0,
                ..colors.fill
            },
            false => colors.fill,
        };
        button::Style {
            background: Some(Background::Color(fill)),
            text_color: colors.login_text(ready),
            border: frame(colors, focused),
            ..button::Style::default()
        }
    }
}

pub fn bare(_: &iced::Theme, _: button::Status) -> button::Style {
    button::Style {
        background: None,
        ..button::Style::default()
    }
}

pub fn tint(color: Color) -> impl Fn(&iced::Theme, svg::Status) -> svg::Style {
    move |_, _| svg::Style { color: Some(color) }
}

#[cfg(test)]
mod tests {
    use iced::Color;
    use iced::border::Radius;
    use iced::widget::{button, text_input};
    use sparky_theme::Skin;

    use super::Colors;

    #[test]
    fn as_cores_vem_do_tema_do_sparky() {
        let skin = Skin::default();
        let colors = Colors::new(skin);
        assert_eq!(colors.clock, skin.accent);
        assert_eq!(colors.icon, skin.bright_foreground);
        assert_eq!(
            colors.fill,
            Color {
                a: 0.2,
                ..skin.background
            }
        );
    }

    #[test]
    fn os_campos_e_o_login_tem_cantos_retos() {
        let colors = Colors::new(Skin::default());
        let theme = Skin::default().theme();
        let campo = super::field(colors)(&theme, text_input::Status::Active);
        let botao = super::login(colors, false, true)(&theme, button::Status::Active);
        assert_eq!(campo.border.radius, Radius::from(0.0));
        assert_eq!(botao.border.radius, Radius::from(0.0));
    }

    #[test]
    fn o_login_fica_apagado_ate_ter_usuario_e_senha() {
        let colors = Colors::new(Skin::default());
        assert_eq!(colors.login_text(false).a, 0.5);
        assert_eq!(colors.login_text(true), colors.button_text);
    }
}
