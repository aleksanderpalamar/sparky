use iced::widget::{button, image, mouse_area, row, svg, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::tray::{TrayIcon, TrayItem};
use crate::ui::style::{HOVER_ALPHA, ICON_SIZE, PADDING_X, SLOT_WIDTH, TEXT_SIZE};

pub fn tray<'a>(skin: &Skin, items: &'a [TrayItem]) -> Element<'a, Message> {
    let icons = items.iter().map(|item| icon_button(skin, item));
    row(icons)
        .height(Length::Fill)
        .padding([0.0, PADDING_X])
        .align_y(Alignment::Center)
        .into()
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

fn icon<'a>(skin: &Skin, item: &'a TrayItem) -> Element<'a, Message> {
    let centered = |element: Element<'a, Message>| {
        iced::widget::container(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    };
    match &item.icon {
        TrayIcon::File(path) if path.extension().is_some_and(|extension| extension == "svg") => {
            centered(svg(path.clone()).width(ICON_SIZE).height(ICON_SIZE).into())
        }
        TrayIcon::File(path) => centered(image(path.clone()).width(ICON_SIZE).height(ICON_SIZE).into()),
        TrayIcon::Pixels { width, height, rgba } => {
            let handle = image::Handle::from_rgba(*width, *height, rgba.clone());
            centered(image(handle).width(ICON_SIZE).height(ICON_SIZE).into())
        }
        TrayIcon::Missing => centered(fallback(skin, item)),
    }
}

fn fallback<'a>(skin: &Skin, item: &'a TrayItem) -> Element<'a, Message> {
    let initial: String = item.title.chars().next().map(String::from).unwrap_or_else(|| "?".to_owned());
    text(initial).size(TEXT_SIZE).color(skin.foreground).into()
}
