use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Space, column, container, float, mouse_area};
use iced::{Element, Length, Vector};
use sparky_theme::Skin;

use super::menu::menu;
use super::style;
use super::tray::tray;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::layout::LIFT;

pub fn view(dock: &Dock) -> Element<'_, Message> {
    let content = match dock.tucked() {
        true => strip(dock.skin),
        false => shown(dock),
    };
    mouse_area(content)
        .on_enter(Message::Hover(true))
        .on_exit(Message::Hover(false))
        .on_move(Message::Pointer)
        .on_release(Message::Release)
        .into()
}

fn shown(dock: &Dock) -> Element<'_, Message> {
    let above = dock
        .menu
        .as_deref()
        .and_then(|id| menu(dock, id))
        .unwrap_or_else(|| Space::new().into());
    let drop = dock.slide.value * (dock.layout().height() + LIFT);
    let body = container(tray(dock))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(iced::Padding::ZERO.bottom(LIFT));
    let sliding = float(body).translate(move |_, _| Vector::new(0.0, drop));
    column![above, sliding]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .into()
}

fn strip(skin: Skin) -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .style(move |_| style::strip(skin))
        .into()
}
