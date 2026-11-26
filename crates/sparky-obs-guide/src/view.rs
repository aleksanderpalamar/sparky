use iced::widget::{button, column, container, row, text};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use crate::app::{Message, Window};
use crate::i18n;

const STEPS: [&str; 3] = ["step-open", "step-enable", "step-apply"];

pub fn view(window: &Window) -> Element<'_, Message> {
    let skin = window.skin;
    let steps = STEPS.iter().fold(
        column![
            text(i18n::text("steps"))
                .size(TEXT_SIZE)
                .color(skin.bright_foreground)
        ]
        .spacing(SPACING / 2.0),
        |steps, step| {
            steps.push(
                text(i18n::text(step))
                    .size(TEXT_SIZE)
                    .color(skin.foreground),
            )
        },
    );
    let understood = button(text(i18n::text("understood")).size(TEXT_SIZE))
        .padding(8)
        .on_press(Message::Understood)
        .style(move |_, status| style::entry(skin, Highlight::Selected, status));
    let never = button(text(i18n::text("never")).size(TEXT_SIZE))
        .padding(8)
        .on_press(Message::Never)
        .style(move |_, status| style::entry(skin, Highlight::Plain, status));
    let content = column![
        text(i18n::text("title"))
            .size(TITLE_SIZE)
            .color(skin.bright_foreground),
        text(i18n::text("explanation"))
            .size(TEXT_SIZE)
            .color(skin.foreground),
        steps,
        row![understood, never].spacing(SPACING),
        text(i18n::text("hint"))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    ]
    .spacing(SPACING);
    container(content)
        .padding(PADDING * 1.5)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
        .into()
}

pub fn panel(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.background)),
        border: Border {
            color: skin.accent,
            width: 2.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::border::Radius;
    use sparky_theme::Skin;

    use super::panel;

    #[test]
    fn o_aviso_tem_cantos_retos_e_borda_de_destaque() {
        let style = panel(Skin::default());
        assert_eq!(style.border.radius, Radius::from(0.0));
        assert_eq!(style.border.color, Skin::default().accent);
    }
}
