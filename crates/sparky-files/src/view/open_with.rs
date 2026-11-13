use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    Space, button, column, container, mouse_area, row, scrollable, stack, text, text_input, toggler,
};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style;

use crate::app::message::Message;
use crate::app::open_with::SEARCH;
use crate::app::state::{Files, OpenWith};
use crate::domain::application::Remember;
use crate::i18n;
use crate::view::open_with_list::list;
use crate::view::style::dialog;

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Some(choice) = files.open_with.as_ref() else {
        return base;
    };
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::ContextDismiss);
    let centered = container(pane(files.skin, choice, files.size.height))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![base, backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn pane(skin: Skin, choice: &OpenWith, height: f32) -> Element<'static, Message> {
    let filename = choice
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| choice.path.display().to_string());
    let search = text_input(&i18n::text("open-with-search"), &choice.query)
        .id(SEARCH)
        .on_input(Message::OpenWithQuery)
        .on_submit(Message::OpenWithConfirm)
        .size(13)
        .padding(8)
        .style(style::input);
    let prompt = text(i18n::with("open-file-with", "file", filename))
        .size(13)
        .width(Length::Fill)
        .align_x(Horizontal::Center);
    let apps = container(scrollable(list(skin, choice)))
        .height((height - 260.0).clamp(120.0, 360.0))
        .width(Length::Fill)
        .padding(4)
        .style(move |_| dialog(skin, skin.background));
    let content = column![
        header(skin, choice.selected.is_some()),
        search,
        prompt,
        apps,
        remember(skin, choice)
    ]
    .spacing(12);
    container(content)
        .width(440)
        .padding(16)
        .style(move |_| dialog(skin, skin.lighter_background))
        .into()
}

fn header(skin: Skin, ready: bool) -> Element<'static, Message> {
    let cancel = button(text(i18n::text("cancel")).size(13))
        .padding([6, 12])
        .on_press(Message::ContextDismiss)
        .style(move |_, status| style::entry(skin, style::Highlight::Plain, status));
    let open = button(text(i18n::text("open")).size(13))
        .padding([6, 12])
        .on_press_maybe(ready.then_some(Message::OpenWithConfirm))
        .style(move |_, status| style::entry(skin, style::Highlight::Selected, status));
    let title = text(i18n::text("open-file"))
        .size(15)
        .width(Length::Fill)
        .align_x(Horizontal::Center);
    row![cancel, title, open].align_y(Vertical::Center).into()
}

fn remember(skin: Skin, choice: &OpenWith) -> Element<'static, Message> {
    let always = choice.remember == Remember::Always;
    let labels = column![
        text(i18n::text("open-with-always")).size(13),
        text(choice.choices.kind.clone())
            .size(11)
            .color(skin.dark_foreground)
    ]
    .width(Length::Fill);
    let switch = toggler(always)
        .on_toggle(Message::OpenWithRemember)
        .style(sparky_ui::switch::switch(skin));
    row![labels, switch].align_y(Vertical::Center).into()
}
