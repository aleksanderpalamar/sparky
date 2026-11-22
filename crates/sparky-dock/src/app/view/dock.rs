use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Space, container, float, stack};
use iced::{Element, Length, Vector};

use super::menu::menu;
use super::style;
use super::tray::tray;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::layout::{LIFT, PEEK};

pub fn view(dock: &Dock) -> Element<'_, Message> {
    match dock.tucked() {
        true => strip(dock),
        false => shown(dock),
    }
}

fn shown(dock: &Dock) -> Element<'_, Message> {
    let layout = dock.layout();
    let rest = layout.rest() as f32;
    let drop = dock.slide.value * rest;
    let body = container(tray(dock))
        .width(Length::Fill)
        .height(rest + layout.rise() as f32)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(iced::Padding::ZERO.bottom(LIFT));
    let sliding = float(body).translate(move |_, _| Vector::new(0.0, drop));
    let above = dock
        .menu
        .as_deref()
        .and_then(|id| menu(dock, id))
        .unwrap_or_else(|| Space::new().into());
    stack![grounded(sliding.into(), 0.0), grounded(above, rest)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn strip(dock: &Dock) -> Element<'_, Message> {
    let skin = dock.skin;
    let count = dock.pinned.resolve(&dock.apps).len();
    let line = container(Space::new())
        .width(dock.layout().size(count).0 as f32)
        .height(PEEK as f32)
        .style(move |_| style::strip(skin));
    grounded(line.into(), 0.0)
}

fn grounded(content: Element<'_, Message>, lift: f32) -> Element<'_, Message> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(iced::Padding::ZERO.bottom(lift))
        .into()
}
