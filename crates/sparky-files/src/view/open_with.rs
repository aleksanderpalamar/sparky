use iced::widget::{Space, button, column, container, mouse_area, scrollable, stack, text};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::{Files, OpenWith};

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Some(choice) = files.open_with.as_ref() else {
        return base;
    };
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::ContextDismiss);
    let centered = container(pane(files.skin, choice, files.size.height))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![base, backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn pane(skin: Skin, choice: &OpenWith, height: f32) -> Element<'static, Message> {
    let filename = choice
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| choice.path.display().to_string());
    let title = text(format!("Abrir {filename} com...")).size(15);
    let apps = choice.apps.iter().map(|app| {
        let id = app.id.clone();
        button(text(app.name.clone()).size(13))
            .on_press(Message::OpenWithChoice(id))
            .width(Length::Fill)
            .style(move |_, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                button::Style {
                    background: hovered.then_some(Background::Color(skin.selection)),
                    text_color: skin.foreground,
                    ..button::Style::default()
                }
            })
            .into()
    });
    let list: Element<'static, Message> = scrollable(column(apps).spacing(4))
        .height((height - 120.0).clamp(80.0, 380.0))
        .into();
    let content = column![
        title,
        list,
        button("Cancelar").on_press(Message::ContextDismiss)
    ]
    .spacing(12);
    container(content)
        .width(420)
        .padding(16)
        .style(move |_| container::Style {
            background: Some(Background::Color(skin.lighter_background)),
            text_color: Some(skin.foreground),
            border: Border {
                color: skin.accent,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
