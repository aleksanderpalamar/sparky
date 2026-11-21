use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    Space, button, checkbox, column, container, mouse_area, opaque, row, scrollable, stack, text,
    text_input,
};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TITLE_SIZE};

use super::picture::picture;
use super::style as look;
use crate::app::message::Message;
use crate::app::state::Dock;
use crate::domain::app::App;
use crate::domain::picker::Picker;
use crate::i18n::text as tr;

const WIDTH: f32 = 520.0;
const LIST_HEIGHT: f32 = 380.0;
const ICON: f32 = 32.0;

pub fn view<'a>(dock: &'a Dock, picker: &'a Picker) -> Element<'a, Message> {
    let skin = dock.skin;
    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .style(move |_| look::backdrop(skin)),
    )
    .on_press(Message::Cancel);
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
        .padding(8)
        .style(style::input);
    let content = column![
        header(skin),
        text(tr("picker-subtitle"))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
        search,
        container(scrollable(list(dock, picker)).spacing(4)).height(LIST_HEIGHT),
        footer(skin, !picker.chosen().is_empty()),
    ]
    .spacing(PADDING);
    container(content)
        .width(WIDTH)
        .padding(PADDING * 1.5)
        .style(move |_| look::dialog(skin))
        .into()
}

fn header(skin: Skin) -> Element<'static, Message> {
    let close = button(text("×").size(TITLE_SIZE))
        .padding([0, 6])
        .on_press(Message::Cancel)
        .style(move |_, status| style::entry(skin, Highlight::Plain, status));
    row![
        text(tr("picker-title"))
            .size(TITLE_SIZE)
            .color(skin.bright_foreground)
            .width(Length::Fill),
        close
    ]
    .align_y(Vertical::Center)
    .into()
}

fn list<'a>(dock: &'a Dock, picker: &'a Picker) -> Element<'a, Message> {
    let visible = picker.visible(&dock.apps);
    if visible.is_empty() {
        return container(text(tr("picker-empty")).color(dock.skin.dark_foreground))
            .width(Length::Fill)
            .padding(PADDING)
            .align_x(Horizontal::Center)
            .into();
    }
    column(visible.into_iter().map(|app| choice(dock, picker, app)))
        .spacing(2)
        .into()
}

fn choice<'a>(dock: &'a Dock, picker: &'a Picker, app: &'a App) -> Element<'a, Message> {
    let skin = dock.skin;
    let pinned = dock.pinned.contains(&app.id);
    let id = app.id.clone();
    let mark = checkbox(pinned || picker.is_chosen(&app.id))
        .on_toggle_maybe((!pinned).then_some(move |_| Message::Toggle(id.clone())))
        .size(16)
        .style(move |_, status| look::check(skin, status));
    let name = match pinned {
        true => column![
            text(app.name.as_str()),
            text(tr("picker-pinned"))
                .size(SMALL_TEXT)
                .color(skin.dark_foreground)
        ]
        .into(),
        false => Element::from(text(app.name.as_str())),
    };
    let line = row![
        picture(dock.icons.get(&app.id), None, ICON),
        name,
        mark,
        Space::new().width(Length::Fill)
    ]
    .spacing(SPACING * 1.5)
    .align_y(Vertical::Center);
    container(line)
        .width(Length::Fill)
        .padding([10, 12])
        .style(move |_| look::card(skin))
        .into()
}

fn footer(skin: Skin, ready: bool) -> Element<'static, Message> {
    let cancel = button(text(tr("picker-cancel")))
        .padding([6, 14])
        .on_press(Message::Cancel)
        .style(move |_, status| style::entry(skin, Highlight::Plain, status));
    let add = button(text(tr("picker-add")))
        .padding([6, 14])
        .on_press_maybe(ready.then_some(Message::Confirm))
        .style(move |_, status| look::primary(skin, status));
    row![Space::new().width(Length::Fill), cancel, add]
        .spacing(SPACING)
        .into()
}
