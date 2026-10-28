use iced::widget::{button, container, mouse_area, row, text};
use iced::{Alignment, Element, Length, window};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::reveal::Expansion;
use crate::domain::tray::TrayItem;
use crate::domain::tray_layout::groups;
use crate::ui::drawer::drawer;
use crate::ui::style::{HOVER_ALPHA, PADDING_X, SLOT_WIDTH, TEXT_SIZE};
use crate::ui::tray_icon::icon;

pub fn tray<'a>(
    skin: &Skin,
    items: &'a [TrayItem],
    window: window::Id,
    progress: f32,
) -> Element<'a, Message> {
    let groups = groups(items);
    let pinned = row(groups
        .pinned
        .into_iter()
        .map(|item| icon_button(skin, item)))
    .height(Length::Fill);
    let mut content = row![].height(Length::Fill).align_y(Alignment::Center);
    if !groups.drawer.is_empty() {
        let extent = groups.drawer.len() as f32 * SLOT_WIDTH;
        let icons = row(groups
            .drawer
            .into_iter()
            .map(|item| icon_button(skin, item)))
        .width(extent)
        .height(Length::Fill);
        let chevron = button(
            container(text("‹").size(TEXT_SIZE + 4.0).color(skin.foreground)).center(Length::Fill),
        )
        .width(SLOT_WIDTH)
        .height(Length::Fill)
        .padding(0)
        .on_press(Message::TrayHover(window, Expansion::Open))
        .style(button::text);
        let area = row![chevron, drawer(icons.into(), extent, progress)].height(Length::Fill);
        content = content.push(
            mouse_area(area)
                .on_enter(Message::TrayHover(window, Expansion::Open))
                .on_exit(Message::TrayHover(window, Expansion::Closed)),
        );
    }
    content.push(pinned).padding([0.0, PADDING_X]).into()
}

fn icon_button<'a>(skin: &Skin, item: &'a TrayItem) -> Element<'a, Message> {
    let skin = *skin;
    let body = button(icon(&skin, item))
        .width(SLOT_WIDTH)
        .height(Length::Fill)
        .padding(0)
        .on_press(Message::TrayActivate(item.address.clone()))
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then(|| skin.fill(HOVER_ALPHA).into()),
                text_color: skin.foreground,
                ..button::Style::default()
            }
        });
    mouse_area(body)
        .on_right_press(Message::TrayMenu(item.address.clone()))
        .into()
}
