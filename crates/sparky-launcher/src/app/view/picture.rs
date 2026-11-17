use iced::Element;
use iced::widget::{column, container, image, svg};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::pictures::Shown;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    Natural,
    Symbolic,
}

pub fn picture(
    skin: Skin,
    icon: Option<&Shown>,
    tint: Tint,
    size: f32,
) -> Element<'static, Message> {
    let shown: Element<'static, Message> = match (icon, tint) {
        (None, _) => column![].into(),
        (Some(Shown::Vector(path)), Tint::Symbolic) => svg(path.clone())
            .width(size)
            .height(size)
            .style(move |_, _| svg::Style {
                color: Some(skin.bright_foreground),
            })
            .into(),
        (Some(Shown::Vector(path)), Tint::Natural) => {
            svg(path.clone()).width(size).height(size).into()
        }
        (Some(Shown::Raster(handle)), _) => image(handle.clone()).width(size).height(size).into(),
    };
    container(shown)
        .width(size)
        .height(size)
        .center(size)
        .into()
}
