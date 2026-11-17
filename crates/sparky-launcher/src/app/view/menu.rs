use iced::widget::scrollable::Direction;
use iced::widget::{Column, Space, button, column, container, row, scrollable, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING};

use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::app::update::MENU;
use crate::app::view::overlay::frame;
use crate::app::view::scrollbar;
use crate::domain::menu::{Item, Spot, fitted, placed};
use crate::domain::mode::Mode;
use crate::i18n;

pub const MENU_ITEM: f32 = 32.0;
const MENU_WIDTH: f32 = 320.0;
const MENU_PADDING: f32 = 6.0;

pub fn menu_height(count: usize, mode: Mode) -> f32 {
    let room = mode.size().1 as f32 - 2.0 * (PADDING + MENU_PADDING);
    fitted(count, MENU_ITEM, room)
}

pub fn layer(launcher: &Launcher, point: (f32, f32), cursor: usize) -> Element<'static, Message> {
    let skin = launcher.skin;
    let items = launcher.menu_items();
    let height = menu_height(items.len(), launcher.mode);
    let (width, tall) = launcher.mode.size();
    let outer = (MENU_WIDTH, height + 2.0 * MENU_PADDING);
    let (x, y) = placed(point, (width as f32, tall as f32), outer);
    let rows = items.iter().enumerate().map(|(index, item)| {
        let name = item.spot.map(|spot| spot_name(launcher, spot));
        menu_row(skin, item, name, index, index == cursor)
    });
    let list = scrollable(Column::with_children(rows))
        .id(MENU)
        .direction(Direction::Vertical(scrollbar::bar()))
        .spacing(scrollbar::GAP)
        .height(height)
        .on_scroll(|viewport| Message::MenuScrolled(viewport.absolute_offset().y))
        .style(move |theme, status| scrollbar::style(skin, theme, status));
    let pane = container(list)
        .padding(MENU_PADDING)
        .width(MENU_WIDTH)
        .style(move |_| frame(skin));
    column![Space::new().height(y), row![Space::new().width(x), pane]].into()
}

fn spot_name(launcher: &Launcher, spot: Spot) -> String {
    match spot {
        Spot::Category(category) => i18n::text(category.label()),
        Spot::Group(index) => launcher
            .library
            .groups
            .get(index)
            .map(|group| group.name.clone())
            .unwrap_or_default(),
    }
}

fn menu_row(
    skin: Skin,
    item: &Item,
    name: Option<String>,
    index: usize,
    current: bool,
) -> Element<'static, Message> {
    let label = match name {
        Some(name) => i18n::with_group(item.label, &name),
        None => i18n::text(item.label),
    };
    button(text(label).size(13.0).color(skin.foreground))
        .width(Length::Fill)
        .height(MENU_ITEM)
        .padding([7.0, 10.0])
        .style(move |_, status| style::entry(skin, Highlight::when(current), status))
        .on_press(Message::Choose(index))
        .into()
}
