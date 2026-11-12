use iced::alignment::Vertical;
use iced::widget::{Column, Space, column, container, image, mouse_area, row, svg, text};
use iced::{Background, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::OpenWith;
use crate::domain::application::{Application, Section};
use crate::i18n;

const ICON: f32 = 24.0;

pub fn list(skin: Skin, choice: &OpenWith) -> Element<'static, Message> {
    let visible = choice.choices.visible(&choice.query);
    if visible.is_empty() {
        return text(i18n::text("open-with-none"))
            .size(13)
            .color(skin.dark_foreground)
            .into();
    }
    let mut rows: Vec<Element<'static, Message>> = Vec::new();
    let mut section = None;
    for (current, app) in visible {
        if section != Some(current) {
            section = Some(current);
            rows.push(heading(skin, current));
        }
        let selected = choice.selected.as_deref() == Some(app.id.as_str());
        rows.push(entry(skin, app, selected));
    }
    Column::with_children(rows).spacing(2).into()
}

fn heading(skin: Skin, section: Section) -> Element<'static, Message> {
    let key = match section {
        Section::Recommended => "open-with-recommended",
        Section::Other => "open-with-others",
    };
    container(text(i18n::text(key)).size(13).color(skin.accent))
        .padding([8, 6])
        .into()
}

fn entry(skin: Skin, app: &Application, selected: bool) -> Element<'static, Message> {
    let line = row![icon(app), text(app.name.clone()).size(13)]
        .spacing(10)
        .align_y(Vertical::Center);
    let cell = container(line)
        .width(Length::Fill)
        .padding([6, 8])
        .style(move |_| container::Style {
            background: selected.then_some(Background::Color(skin.selection)),
            text_color: Some(skin.foreground),
            ..container::Style::default()
        });
    mouse_area(cell)
        .on_press(Message::OpenWithSelect(app.id.clone()))
        .on_double_click(Message::OpenWithChoice(app.id.clone()))
        .into()
}

fn icon(app: &Application) -> Element<'static, Message> {
    match &app.icon {
        Some(path) if path.extension().is_some_and(|extension| extension == "svg") => {
            svg(svg::Handle::from_path(path))
                .width(ICON)
                .height(ICON)
                .into()
        }
        Some(path) => image(path.clone()).width(ICON).height(ICON).into(),
        None => column![Space::new().width(ICON).height(ICON)].into(),
    }
}
