use iced::widget::{Space, container, image, svg};
use iced::{Color, Element};
use sparky_launcher::app::pictures::Shown;

use crate::app::message::Message;

pub fn picture(
    icon: Option<&Shown>,
    tint: Option<Color>,
    size: f32,
    opacity: f32,
) -> Element<'static, Message> {
    let shown: Element<'static, Message> = match icon {
        None => Space::new().width(size).height(size).into(),
        Some(Shown::Vector(path)) => svg(path.clone())
            .width(size)
            .height(size)
            .opacity(opacity)
            .style(move |_, _| svg::Style { color: tint })
            .into(),
        Some(Shown::Raster(handle)) => image(handle.clone())
            .width(size)
            .height(size)
            .opacity(opacity)
            .into(),
    };
    container(shown)
        .width(size)
        .height(size)
        .center(size)
        .into()
}
