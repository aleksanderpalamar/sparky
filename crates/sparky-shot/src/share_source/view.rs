use super::{Message, Picker};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;

pub(super) fn list_id() -> iced::widget::Id {
    iced::widget::Id::new("sparky-share-sources")
}

pub(super) fn view(picker: &Picker) -> Element<'_, Message> {
    let skin = picker.skin;
    let mut sources = column![].spacing(6);
    for (position, (index, source)) in picker.selection.visible().into_iter().enumerate() {
        let selected = position == picker.selection.selected;
        sources = sources.push(
            button(text(source.label.clone()).color(skin.foreground))
                .width(Length::Fill)
                .padding(12)
                .on_press(Message::Pick(index))
                .style(move |_, status| entry(skin, selected, status)),
        );
    }
    let content = column![
        text("Compartilhar a tela")
            .size(22)
            .color(skin.bright_foreground),
        text("Escolha o monitor ou a janela que o aplicativo poderá capturar.")
            .color(skin.foreground),
        text_input("Buscar fonte...", &picker.selection.query).on_input(Message::Search),
        scrollable(sources).id(list_id()).height(Length::Fill),
        row![
            text("↑ ↓ selecionar · Enter compartilhar · Esc cancelar")
                .size(12)
                .color(skin.foreground),
            button("Cancelar")
                .on_press(Message::Cancel)
                .style(move |_, status| entry(skin, false, status))
        ]
        .spacing(12),
    ]
    .spacing(12);
    container(content)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
        .into()
}

fn panel(skin: Skin) -> container::Style {
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

fn entry(skin: Skin, selected: bool, status: button::Status) -> button::Style {
    let active = selected || matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(if active {
            skin.lighter_background
        } else {
            skin.background
        })),
        text_color: skin.foreground,
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn painel_opaco_e_cantos_retos_seguem_o_tema() {
        for colors in [
            "mode = \"dark\"",
            "mode = \"light\"\nbackground = \"#ffffff\"\nforeground = \"#111111\"",
        ] {
            let skin = Skin::new(sparky_theme::Palette::from_colors(colors));
            assert_eq!(
                panel(skin).background,
                Some(Background::Color(skin.background))
            );
            assert_eq!(panel(skin).border.radius, iced::border::Radius::from(0.0));
            assert_eq!(
                entry(skin, false, button::Status::Active).border.radius,
                iced::border::Radius::from(0.0)
            );
        }
    }
}
