use iced::widget::{button, column, container, rule, text};
use iced::{Border, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::tray::MenuEntry;
use crate::ui::style::{HOVER_ALPHA, MENU_ROW, PADDING_X, TEXT_SIZE};

pub fn menu<'a>(skin: &Skin, entries: &'a [MenuEntry]) -> Element<'a, Message> {
    let skin = *skin;
    let rows = entries.iter().map(|entry| row_for(&skin, entry));
    container(column(rows).padding(6))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(skin.background.into()),
            border: Border {
                color: skin.accent,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn row_for<'a>(skin: &Skin, entry: &'a MenuEntry) -> Element<'a, Message> {
    if entry.separator {
        return container(rule::horizontal(1)).height(MENU_ROW).center_y(MENU_ROW).into();
    }
    let skin = *skin;
    let color = match entry.enabled {
        true => skin.foreground,
        false => skin.dark_foreground,
    };
    let label = text(entry.label.as_str()).size(TEXT_SIZE).color(color);
    let item = button(container(label).center_y(Length::Fill))
        .width(Length::Fill)
        .height(MENU_ROW)
        .padding([0.0, PADDING_X])
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then(|| skin.fill(HOVER_ALPHA * 2.0).into()),
                text_color: color,
                ..button::Style::default()
            }
        });
    match entry.enabled {
        true => item.on_press(Message::TrayChoose(entry.id)).into(),
        false => item.into(),
    }
}
