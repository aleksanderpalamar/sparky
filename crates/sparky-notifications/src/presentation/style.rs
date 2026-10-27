use iced::{
    Border, Element,
    widget::{button, container},
};
use sparky_theme::Skin;

pub fn surface(skin: Skin) -> container::Style {
    container::Style {
        background: Some(
            iced::Color {
                a: 1.0,
                ..skin.background
            }
            .into(),
        ),
        text_color: Some(skin.foreground),
        border: Border {
            color: skin.accent,
            width: 2.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn action<'a, M: Clone + 'a>(
    skin: &Skin,
    label: impl Into<String>,
    message: M,
) -> Element<'a, M> {
    let skin = *skin;
    button(iced::widget::text(label.into()).size(12))
        .padding([6, 8])
        .on_press(message)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(skin.fill(if hovered { 0.16 } else { 0.08 }).into()),
                text_color: skin.foreground,
                ..Default::default()
            }
        })
        .into()
}
