use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Space, button, column, container, mouse_area, row, stack, text};
use iced::{Element, Length, mouse};
use sparky_theme::Skin;

use super::picture::picture;
use super::style;
use crate::adapters::icons::ADD;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::app::{animate, pointer};
use crate::domain::app::App;
use crate::domain::layout::{DIVIDER, INSET_X, INSET_Y, Layout, MARK, MARK_GAP, OUTLINE};
use crate::domain::presence::{Presence, presence};

const DRAGGED: f32 = 0.45;

pub fn tray(dock: &Dock) -> Element<'_, Message> {
    let skin = dock.skin;
    let corner = dock.corner();
    let layout = dock.layout();
    let order = pointer::order(dock);
    let mut scales = animate::scales(dock);
    scales.resize(order.len() + 1, 1.0);
    let dragged = dock
        .drag
        .as_ref()
        .filter(|drag| drag.moved)
        .map(|drag| drag.id.as_str());
    let apps = order
        .iter()
        .filter_map(|id| dock.apps.iter().find(|app| &app.id == id))
        .zip(scales.iter().copied())
        .map(|(app, scale)| launcher(dock, layout, app, scale, dragged == Some(app.id.as_str())));
    let plus = add(dock, layout, scales.last().copied().unwrap_or(1.0));
    let icons = row(apps.chain([divider(skin, layout), plus]))
        .spacing(layout.gap())
        .align_y(Vertical::Bottom);
    let backdrop = container(Space::new())
        .width(layout.tray_width(&scales))
        .height(layout.height())
        .style(move |_| style::tray(skin, corner));
    let surface = dock.surface.map_or(0.0, |geometry| geometry.size.0 as f32);
    let left = layout.tray_left(&scales, surface);
    stack![
        placed(backdrop.into(), left, 0.0),
        placed(icons.into(), left + INSET_X + OUTLINE, INSET_Y + OUTLINE)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn placed(content: Element<'_, Message>, left: f32, lift: f32) -> Element<'_, Message> {
    container(row![Space::new().width(left.max(0.0)), content])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Vertical::Bottom)
        .padding(iced::Padding::ZERO.bottom(lift))
        .into()
}

fn launcher<'a>(
    dock: &'a Dock,
    layout: Layout,
    app: &'a App,
    scale: f32,
    dragged: bool,
) -> Element<'a, Message> {
    let skin = dock.skin;
    let size = layout.slot * scale;
    let glyph = layout.glyph() * scale;
    let opacity = if dragged { DRAGGED } else { 1.0 };
    let face = match dock.icons.get(&app.id) {
        Some(icon) => picture(Some(icon), None, glyph, opacity),
        None => initial(&app.name, glyph),
    };
    let tile = mouse_area(container(face).center(size))
        .on_press(Message::Press(app.id.clone()))
        .on_right_press(Message::Menu(app.id.clone()))
        .interaction(mouse::Interaction::Pointer);
    column![tile, mark(skin, presence(app, &dock.windows))]
        .spacing(MARK_GAP)
        .align_x(Horizontal::Center)
        .into()
}

fn add(dock: &Dock, layout: Layout, scale: f32) -> Element<'_, Message> {
    let skin = dock.skin;
    let size = layout.slot * scale;
    let glyph = layout.glyph() * scale;
    let face = match dock.icons.get(ADD) {
        Some(icon) => picture(Some(icon), Some(skin.accent), glyph, 1.0),
        None => text("+").size(glyph).color(skin.accent).into(),
    };
    let tile = button(container(face).center(size))
        .width(size)
        .height(size)
        .padding(0)
        .on_press(Message::OpenPicker)
        .style(move |_, status| style::add(skin, status));
    column![tile, Space::new().height(MARK)]
        .spacing(MARK_GAP)
        .into()
}

fn divider(skin: Skin, layout: Layout) -> Element<'static, Message> {
    let line = container(Space::new().width(DIVIDER).height(layout.divider()))
        .style(move |_| style::divider(skin));
    column![
        line,
        Space::new().height(MARK + MARK_GAP + (layout.slot - layout.divider()) / 2.0)
    ]
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
