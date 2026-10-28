use iced::widget::{
    Space, button, column, container, mouse_area, row, rule, scrollable, stack, text,
};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::context::{self, Row};

const WIDTH: f32 = 280.0;
const ITEM_HEIGHT: f32 = 32.0;

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    if files.open_with.is_some() {
        return crate::view::open_with::layer(files, base);
    }
    if files.details.is_some() {
        return crate::view::properties::layer(files, base);
    }
    let Some(menu) = &files.context else {
        return base;
    };
    let entries = context::entries(menu.target, files.clip.is_some(), !files.listing.is_empty());
    let height = context::height(&entries).min((files.size.height - 16.0).max(32.0));
    let (x, y) = context::position(
        (menu.point.x, menu.point.y),
        (files.size.width, files.size.height),
        (WIDTH, height),
    );
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::ContextDismiss)
        .on_right_press(Message::ContextDismiss);
    let pane = menu_pane(files.skin, &entries, height);
    let positioned = column![Space::new().height(y), row![Space::new().width(x), pane],];
    stack![base, backdrop, positioned]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn menu_pane(skin: Skin, entries: &[Row], height: f32) -> Element<'static, Message> {
    let rows = entries.iter().copied().map(|entry| menu_row(skin, entry));
    let body = scrollable(column(rows).padding(6)).height(height);
    container(body)
        .width(WIDTH)
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
        text(label).size(13.0).color(color).width(Length::Fill),
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
