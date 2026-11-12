use iced::widget::toggler;
use iced::{Color, Theme};
use sparky_theme::Skin;

const NORMAL_FILL: f32 = 0.04;
const HOVER_FILL: f32 = 0.08;
const SELECTED_FILL: f32 = 0.18;
const NORMAL_BORDER: f32 = 0.4;

pub fn switch(skin: Skin) -> impl Fn(&Theme, toggler::Status) -> toggler::Style {
    move |_, status| {
        let (checked, hovered) = match status {
            toggler::Status::Active { is_toggled } | toggler::Status::Disabled { is_toggled } => {
                (is_toggled, false)
            }
            toggler::Status::Hovered { is_toggled } => (is_toggled, true),
        };
        let track = match (checked, hovered) {
            (true, _) => SELECTED_FILL,
            (false, true) => HOVER_FILL,
            (false, false) => NORMAL_FILL,
        };
        let (edge, knob) = match checked {
            true => (skin.foreground, skin.foreground),
            false => (skin.fill(NORMAL_BORDER), skin.dark_foreground),
        };
        toggler::Style {
            background: skin.fill(track).into(),
            background_border_width: 1.0,
            background_border_color: edge,
            foreground: knob.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            border_radius: Some(0.0.into()),
            padding_ratio: 0.14,
            text_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::toggler::Status;
    use iced::{Background, Theme};
    use sparky_theme::Skin;

    use super::switch;

    #[test]
    fn ligado_usa_a_cor_do_texto_e_o_canto_reto() {
        let skin = Skin::default();
        let look = switch(skin);
        let ligado = look(&Theme::Dark, Status::Active { is_toggled: true });
        let desligado = look(&Theme::Dark, Status::Active { is_toggled: false });
        assert_eq!(ligado.foreground, Background::Color(skin.foreground));
        assert_eq!(
            desligado.foreground,
            Background::Color(skin.dark_foreground)
        );
        assert_eq!(ligado.border_radius, Some(0.0.into()));
    }
}
