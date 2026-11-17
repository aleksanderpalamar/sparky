use iced::widget::scrollable::{self, Scrollbar, Status};
use iced::{Background, Border, Theme};
use sparky_theme::Skin;

const WIDTH: f32 = 6.0;
pub const GAP: f32 = 6.0;

pub fn bar() -> Scrollbar {
    Scrollbar::new().width(WIDTH).scroller_width(WIDTH)
}

pub fn style(skin: Skin, theme: &Theme, status: Status) -> scrollable::Style {
    let standard = scrollable::default(theme, status);
    let active = matches!(
        status,
        Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | Status::Dragged {
            is_vertical_scrollbar_dragged: true,
            ..
        }
    );
    let color = match active {
        true => skin.accent,
        false => skin.muted,
    };
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(color),
            border: Border::default(),
        },
    };
    scrollable::Style {
        vertical_rail: rail,
        ..standard
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::scrollable::Status;
    use iced::{Background, Theme};
    use sparky_theme::Skin;

    use super::style;

    fn cor(status: Status) -> Background {
        style(Skin::default(), &Theme::Dark, status)
            .vertical_rail
            .scroller
            .background
    }

    #[test]
    fn a_barra_fica_discreta_e_ganha_a_cor_de_destaque_ao_ser_usada() {
        let skin = Skin::default();
        let parada = Status::Active {
            is_horizontal_scrollbar_disabled: true,
            is_vertical_scrollbar_disabled: false,
        };
        let arrastada = Status::Dragged {
            is_horizontal_scrollbar_dragged: false,
            is_vertical_scrollbar_dragged: true,
            is_horizontal_scrollbar_disabled: true,
            is_vertical_scrollbar_disabled: false,
        };
        assert_eq!(cor(parada), Background::Color(skin.muted));
        assert_eq!(cor(arrastada), Background::Color(skin.accent));
    }
}
