use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Space, button, column, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::menu::menu;
use super::picture::picture;
use super::style;
use crate::adapters::icons::ADD;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::app::App;
use crate::domain::layout::{DIVIDER, INSET_X, INSET_Y, LIFT, Layout, MARK, MARK_GAP, Visibility};
use crate::domain::presence::Presence;

pub fn view(dock: &Dock) -> Element<'_, Message> {
    let content = match dock.visibility() {
        Visibility::Hidden => strip(dock.skin),
        Visibility::Shown => shown(dock),
    };
    mouse_area(content)
        .on_enter(Message::Hover(true))
        .on_exit(Message::Hover(false))
        .into()
}

fn shown(dock: &Dock) -> Element<'_, Message> {
    let tray = tray(dock);
    let above = dock
        .menu
        .as_deref()
        .and_then(|id| menu(dock, id))
        .unwrap_or_else(|| Space::new().into());
    container(column![above, tray].align_x(Horizontal::Center))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(iced::Padding::ZERO.bottom(LIFT))
        .into()
}

fn tray(dock: &Dock) -> Element<'_, Message> {
    let skin = dock.skin;
    let layout = dock.layout();
    let entries = dock.entries();
    let count = entries.len();
    let apps = entries
        .into_iter()
        .map(|(app, presence)| launcher(dock, layout, app, presence));
    let controls = [divider(skin, layout), add(dock, layout)];
    let (width, height) = layout.size(count);
    container(row(apps.chain(controls)).spacing(layout.gap()))
        .padding([INSET_Y, INSET_X])
        .width(width as f32)
        .height(height as f32)
        .style(move |_| style::tray(skin))
        .into()
}

fn launcher<'a>(
    dock: &'a Dock,
    layout: Layout,
    app: &'a App,
    presence: Presence,
) -> Element<'a, Message> {
    let skin = dock.skin;
    let face = match dock.icons.get(&app.id) {
        Some(icon) => picture(Some(icon), None, layout.glyph()),
        None => initial(&app.name, layout.glyph()),
    };
    let tile = button(container(face).center(layout.slot))
        .width(layout.slot)
        .height(layout.slot)
        .padding(0)
        .on_press(Message::Start(app.id.clone()))
        .style(move |_, status| style::slot(skin, presence, status));
    let tile = mouse_area(tile).on_right_press(Message::Menu(app.id.clone()));
    column![tile, mark(skin, presence)]
        .spacing(MARK_GAP)
        .align_x(Horizontal::Center)
        .into()
}

fn add(dock: &Dock, layout: Layout) -> Element<'_, Message> {
    let skin = dock.skin;
    let face = match dock.icons.get(ADD) {
        Some(icon) => picture(Some(icon), Some(skin.accent), layout.glyph()),
        None => text("+").size(layout.glyph()).color(skin.accent).into(),
    };
    let tile = button(container(face).center(layout.slot))
        .width(layout.slot)
        .height(layout.slot)
        .padding(0)
        .on_press(Message::OpenPicker)
        .style(move |_, status| style::add(skin, status));
    column![tile, Space::new().height(MARK)]
        .spacing(MARK_GAP)
        .into()
}

fn divider(skin: Skin, layout: Layout) -> Element<'static, Message> {
    container(Space::new().width(DIVIDER).height(layout.divider()))
        .style(move |_| style::divider(skin))
        .into()
}

fn strip(skin: Skin) -> Element<'static, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .style(move |_| style::strip(skin))
        .into()
}

fn mark(skin: Skin, presence: Presence) -> Element<'static, Message> {
    let Some(color) = style::mark(skin, presence) else {
        return Space::new().width(MARK).height(MARK).into();
    };
    container(Space::new().width(MARK).height(MARK))
        .style(move |_| style::dot(color, MARK))
        .into()
}

fn initial(name: &str, size: f32) -> Element<'static, Message> {
    let letter: String = name
        .chars()
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    text(letter).size(size * 0.6).into()
}
