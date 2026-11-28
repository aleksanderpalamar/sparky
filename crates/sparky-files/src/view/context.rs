use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Background, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::context::{self, Row};
use crate::domain::tag_color::TagColor;
use crate::view::tag_palette::palette;

const WIDTH: f32 = 280.0;
const ITEM_HEIGHT: f32 = 32.0;

pub struct Menu {
    entries: Vec<Row>,
    tag: Option<TagColor>,
}

impl Menu {
    pub fn size(&self, room: f32) -> (f32, f32) {
        (
            WIDTH,
            context::height(&self.entries).min((room - 16.0).max(32.0)),
        )
    }
}

pub fn menu(files: &Files) -> Option<Menu> {
    let menu = files.context.as_ref()?;
    let tag = match menu.target {
        context::Target::Bookmark(index) => files
            .bookmarks
            .items
            .get(index)
            .and_then(|bookmark| bookmark.color),
        _ => None,
    };
    Some(Menu {
        entries: files.menu_rows(menu.target),
        tag,
    })
}

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    if files.open_with.is_some() {
        return crate::view::open_with::layer(files, base);
    }
    if files.details.is_some() {
        return crate::view::properties::layer(files, base);
    }
    base
}

pub fn popup(files: &Files) -> Element<'static, Message> {
    let Some(menu) = menu(files) else {
        return Space::new().into();
    };
    let (_, height) = menu.size(files.size.height);
    menu_pane(files.skin, &menu.entries, height, menu.tag)
}

fn menu_pane(
    skin: Skin,
    entries: &[Row],
    height: f32,
    tag: Option<TagColor>,
) -> Element<'static, Message> {
    let rows = entries.iter().copied().map(|entry| match entry {
        Row::Palette => palette(skin, tag),
        entry => menu_row(skin, entry),
    });
    let body = scrollable(column(rows).padding(6)).height(height);
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| libsparky::look::floating(skin))
        .into()
}

fn menu_row(skin: Skin, entry: Row) -> Element<'static, Message> {
    let Row::Item {
        action,
        label,
        shortcut,
        enabled,
    } = entry
    else {
        return container(rule::horizontal(1))
            .height(10)
            .center_y(10)
            .into();
    };
    let color = if enabled {
        skin.foreground
    } else {
        skin.dark_foreground
    };
    let line = row![
        text(crate::i18n::text(label))
            .size(13.0)
            .color(color)
            .width(Length::Fill),
        text(shortcut).size(11.0).color(skin.dark_foreground),
    ]
    .align_y(iced::alignment::Vertical::Center);
    let item = button(container(line).height(Length::Fill).center_y(Length::Fill))
        .width(Length::Fill)
        .height(ITEM_HEIGHT)
        .padding([0.0, 10.0])
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then_some(Background::Color(skin.selection)),
                text_color: color,
                ..button::Style::default()
            }
        });
    if enabled {
        item.on_press(Message::ContextAction(action)).into()
    } else {
        item.into()
    }
}
