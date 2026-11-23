use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    Space, button, checkbox, column, container, mouse_area, opaque, row, scrollable, stack, text,
    text_input,
};
use iced::{Element, Font, Length, font};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, FONT};

use super::picture::picture;
use super::style as look;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::app::App;
use crate::domain::picker::Picker;
use crate::i18n::{counted, text as tr};

const WIDTH: f32 = 590.0;
const ROW_WIDTH: f32 = 495.0;
const LIST_HEIGHT: f32 = 384.0;
const BADGE: f32 = 34.0;
const ICON: f32 = 30.0;
const TITLE: f32 = 23.0;
const BODY: f32 = 14.0;
const NAME: f32 = 15.0;
const NOTE: f32 = 13.0;
const CHECK: f32 = 18.0;
const STRONG: Font = Font {
    weight: font::Weight::Semibold,
    ..FONT
};

pub fn view<'a>(dock: &'a Dock, picker: &'a Picker) -> Element<'a, Message> {
    let backdrop =
        mouse_area(Space::new().width(Length::Fill).height(Length::Fill)).on_press(Message::Cancel);
    let dialog = container(opaque(pane(dock, picker)))
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill);
    stack![backdrop, dialog].into()
}

fn pane<'a>(dock: &'a Dock, picker: &'a Picker) -> Element<'a, Message> {
    let skin = dock.skin;
    let search = text_input(&tr("picker-search"), picker.query())
        .id(PROMPT)
        .on_input(Message::Search)
        .size(BODY)
        .padding([12, 16])
        .style(style::input);
    let content = column![
        header(skin),
        text(tr("picker-subtitle"))
            .size(BODY)
            .color(skin.dark_foreground),
        search,
        container(scrollable(list(dock, picker))).height(LIST_HEIGHT),
        footer(skin, picker.chosen().len(), picker.full(dock.room())),
    ]
    .spacing(16);
    container(content)
        .width(WIDTH)
        .padding([24, 28])
        .style(move |_| look::dialog(skin))
        .into()
}

fn header(skin: Skin) -> Element<'static, Message> {
    let close = button(text("×").size(25).color(skin.accent))
        .padding([0, 6])
        .on_press(Message::Cancel)
        .style(move |_, status| style::entry(skin, style::Highlight::Plain, status));
    let title = text(tr("picker-title"))
        .size(TITLE)
        .font(STRONG)
        .color(skin.bright_foreground)
        .width(Length::Fill);
    row![title, close].align_y(Vertical::Center).into()
}

fn list<'a>(dock: &'a Dock, picker: &'a Picker) -> Element<'a, Message> {
    let visible = picker.visible(&dock.apps);
    if visible.is_empty() {
        return container(
            text(tr("picker-empty"))
                .size(BODY)
                .color(dock.skin.dark_foreground),
        )
        .width(ROW_WIDTH)
        .padding(16)
        .align_x(Horizontal::Center)
        .into();
    }
    column(visible.into_iter().map(|app| choice(dock, picker, app)))
        .spacing(3)
        .into()
}

fn choice<'a>(dock: &'a Dock, picker: &'a Picker, app: &'a App) -> Element<'a, Message> {
    let skin = dock.skin;
    let pinned = dock.pinned.contains(&app.id);
    let chosen = picker.is_chosen(&app.id);
    let locked = pinned || (!chosen && picker.full(dock.room()));
    let id = app.id.clone();
    let mark = checkbox(pinned || chosen)
        .on_toggle_maybe((!locked).then_some(move |_| Message::Toggle(id.clone())))
        .size(CHECK)
        .style(move |_, status| look::check(skin, status));
    let label = text(app.name.as_str())
        .size(NAME)
        .color(skin.bright_foreground);
    let name: Element<'a, Message> = match pinned {
        true => column![
            label,
            text(tr("picker-pinned"))
                .size(NOTE)
                .color(skin.dark_foreground)
        ]
        .into(),
        false => label.into(),
    };
    let badge = container(picture(dock.icons.get(&app.id), None, ICON, 1.0))
        .center(BADGE)
        .style(move |_| look::badge(skin));
    let line = row![badge, name, mark]
        .spacing(14)
        .align_y(Vertical::Center);
    container(line)
        .width(ROW_WIDTH)
        .padding([14, 16])
        .style(move |_| look::card(skin))
        .into()
}

fn footer(skin: Skin, chosen: usize, full: bool) -> Element<'static, Message> {
    let summary = match full {
        true => tr("picker-full"),
        false => counted("picker-count", chosen),
    };
    let status = text(summary)
        .size(NOTE)
        .color(skin.dark_foreground)
        .width(Length::Fill);
    let cancel = button(text(tr("picker-cancel")).size(BODY))
        .padding([12, 22])
        .on_press(Message::Cancel)
        .style(move |_, status| look::secondary(skin, status));
    let add = button(text(tr("picker-add")).size(BODY).font(STRONG))
        .padding([12, 18])
        .on_press_maybe((chosen > 0).then_some(Message::Confirm))
        .style(move |_, status| look::primary(skin, status));
    row![status, cancel, add]
        .spacing(12)
        .align_y(Vertical::Center)
        .into()
}
