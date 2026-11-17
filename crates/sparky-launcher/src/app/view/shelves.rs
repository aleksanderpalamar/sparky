use iced::alignment::Horizontal;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::Wrapping;
use iced::widget::{Row, column, container, mouse_area, scrollable, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, SMALL_TEXT};

use crate::app::message::Message;
use crate::app::pictures::Shown;
use crate::app::state::Launcher;
use crate::app::view::picture::{Tint, picture};
use crate::domain::shelf::Shelf;
use crate::domain::symbols::{ADD, key};
use crate::i18n;

const BUTTON_WIDTH: f32 = 92.0;
const SYMBOL_SIZE: f32 = 20.0;

pub fn bar(launcher: &Launcher) -> Element<'_, Message> {
    let shelves = launcher
        .shelves()
        .into_iter()
        .map(|shelf| shelf_button(launcher, shelf));
    let add = mouse_area(face(
        launcher.skin,
        launcher.icons.get(ADD),
        i18n::text("shelf-add-group"),
        Highlight::Plain,
    ))
    .on_press(Message::AddGroup)
    .into();
    let line = Row::with_children(shelves.chain(std::iter::once(add))).spacing(4.0);
    let strip = scrollable(line).direction(Direction::Horizontal(Scrollbar::hidden()));
    container(strip).center_x(Length::Fill).into()
}

fn shelf_button(launcher: &Launcher, shelf: Shelf) -> Element<'static, Message> {
    let label = match shelf {
        Shelf::Home => i18n::text("shelf-home"),
        Shelf::Category(category) => i18n::text(category.label()),
        Shelf::Group(index) => launcher
            .library
            .groups
            .get(index)
            .map(|group| group.name.clone())
            .unwrap_or_default(),
    };
    let highlight = Highlight::when(shelf == launcher.shelf);
    let button = mouse_area(face(
        launcher.skin,
        launcher.icons.get(&key(shelf)),
        label,
        highlight,
    ))
    .on_press(Message::ShowShelf(shelf));
    match shelf {
        Shelf::Group(index) => button.on_right_press(Message::GroupMenu(index)).into(),
        _ => button.into(),
    }
}

fn face(
    skin: Skin,
    icon: Option<&Shown>,
    label: String,
    highlight: Highlight,
) -> Element<'static, Message> {
    let caption = text(label)
        .size(SMALL_TEXT)
        .color(skin.foreground)
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .wrapping(Wrapping::None);
    let body = column![picture(skin, icon, Tint::Symbolic, SYMBOL_SIZE), caption]
        .align_x(Horizontal::Center)
        .spacing(4.0);
    container(body)
        .width(BUTTON_WIDTH)
        .padding([6.0, 4.0])
        .clip(true)
        .style(move |_| style::row(skin, highlight))
        .into()
}
