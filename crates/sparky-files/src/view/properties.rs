use chrono::{DateTime, Local};
use iced::widget::{Space, button, column, container, mouse_area, row, stack, text};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Kind;
use crate::domain::properties::Properties;
use crate::domain::size::human;

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Some(details) = files.details.as_ref() else {
        return base;
    };
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::ContextDismiss);
    let centered = container(pane(files.skin, details))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![base, backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn pane(skin: Skin, details: &Properties) -> Element<'static, Message> {
    let name = details
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| details.path.display().to_string());
    let kind = match details.kind {
        Kind::Directory => "Pasta",
        Kind::File => "Arquivo",
    };
    let size = match details.kind {
        Kind::Directory => "—".to_owned(),
        Kind::File => human(details.size),
    };
    let modified = details
        .modified
        .map(|time| {
            DateTime::<Local>::from(time)
                .format("%d/%m/%Y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "Não informado".to_owned());
    let heading = row![
        text("Propriedades").size(16).width(Length::Fill),
        button("×").on_press(Message::ContextDismiss),
    ];
    let content = column![
        heading,
        field(skin, "Nome", name),
        field(skin, "Tipo", kind.to_owned()),
        field(skin, "Caminho", details.path.display().to_string()),
        field(skin, "Tamanho", size),
        field(skin, "Modificado", modified),
        field(skin, "Permissões", format!("{:04o}", details.permissions)),
    ]
    .spacing(10);
    container(content)
        .width(480)
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

fn field(skin: Skin, label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).size(12).color(skin.dark_foreground).width(110),
        text(value)
            .size(13)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
    ]
    .spacing(8)
    .into()
}
