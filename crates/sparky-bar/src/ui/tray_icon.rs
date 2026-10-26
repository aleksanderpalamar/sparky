use iced::widget::{container, image, svg, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::indicator::Status;
use crate::domain::tray::{TrayIcon, TrayItem};
use crate::ui::style::{ICON_SIZE, TEXT_SIZE};

pub fn icon<'a>(skin: &Skin, item: &'a TrayItem) -> Element<'a, Message> {
    let color = match item.status {
        Status::Attention => skin.urgent,
        _ => skin.foreground,
    };
    let body: Element<'a, Message> = match &item.icon {
        TrayIcon::Symbolic(path) => svg(path.clone())
            .width(ICON_SIZE)
            .height(ICON_SIZE)
            .style(move |_, _| svg::Style { color: Some(color) })
            .into(),
        TrayIcon::File(path) if path.extension().is_some_and(|extension| extension == "svg") => {
            svg(path.clone()).width(ICON_SIZE).height(ICON_SIZE).into()
        }
        TrayIcon::File(path) => image(path.clone())
            .width(ICON_SIZE)
            .height(ICON_SIZE)
            .into(),
        TrayIcon::Pixels {
            width,
            height,
            rgba,
        } => {
            let handle = image::Handle::from_rgba(*width, *height, rgba.clone());
            image(handle).width(ICON_SIZE).height(ICON_SIZE).into()
        }
        TrayIcon::Missing => {
            let initial = item.title.chars().next().unwrap_or('?').to_string();
            text(initial).size(TEXT_SIZE).color(color).into()
        }
    };
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
}
