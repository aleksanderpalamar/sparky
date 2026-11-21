use iced::alignment::Horizontal;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::picture::picture;
use super::style;
use crate::adapters::icons::ADD;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::app::App;
use crate::domain::layout::{DIVIDER, GAP, GLYPH, INSET, MARK, MARK_GAP, SLOT};
use crate::domain::presence::Presence;

pub fn view(dock: &Dock) -> Element<'_, Message> {
    let skin = dock.skin;
    let apps = dock
        .entries()
        .into_iter()
        .map(|(app, presence)| launcher(dock, app, presence));
    let controls = [divider(skin), add(dock)];
    let slots = row(apps.chain(controls)).spacing(GAP);
    container(slots)
        .padding(INSET)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| style::tray(skin))
        .into()
}

fn launcher<'a>(dock: &'a Dock, app: &'a App, presence: Presence) -> Element<'a, Message> {
    let skin = dock.skin;
    let face = match dock.icons.get(&app.id) {
        Some(icon) => picture(Some(icon), None, GLYPH),
        None => initial(&app.name),
    };
    let tile = button(container(face).center(SLOT))
        .width(SLOT)
        .height(SLOT)
        .padding(0)
        .on_press(Message::Start(app.id.clone()))
        .style(move |_, status| style::slot(skin, presence, status));
    column![tile, mark(skin, presence)]
        .spacing(MARK_GAP)
        .align_x(Horizontal::Center)
        .into()
}

fn add(dock: &Dock) -> Element<'_, Message> {
    let skin = dock.skin;
    let face = match dock.icons.get(ADD) {
        Some(icon) => picture(Some(icon), Some(skin.accent), GLYPH),
        None => text("+").size(GLYPH).color(skin.accent).into(),
    };
    let tile = button(container(face).center(SLOT))
        .width(SLOT)
        .height(SLOT)
        .padding(0)
        .on_press(Message::OpenPicker)
        .style(move |_, status| style::slot(skin, Presence::Closed, status));
    column![tile, Space::new().height(MARK)]
        .spacing(MARK_GAP)
        .into()
}

fn divider(skin: Skin) -> Element<'static, Message> {
    let line = container(Space::new().width(DIVIDER).height(SLOT * 0.6))
        .style(move |_| style::divider(skin));
    container(line).center_y(SLOT).into()
}

fn mark(skin: Skin, presence: Presence) -> Element<'static, Message> {
    let Some(color) = style::mark(skin, presence) else {
        return Space::new().width(MARK).height(MARK).into();
    };
    container(Space::new().width(MARK).height(MARK))
        .style(move |_| style::dot(color, MARK))
        .into()
}

fn initial(name: &str) -> Element<'static, Message> {
    let letter: String = name
        .chars()
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    text(letter).size(GLYPH * 0.6).into()
}
