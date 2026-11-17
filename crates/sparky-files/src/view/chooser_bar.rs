use iced::alignment::Vertical;
use iced::widget::{Space, button, container, pick_list, row, text, text_input};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};

use crate::app::chooser::{Chooser, ChooserEvent, NAME_INPUT};
use crate::app::message::Message;
use crate::i18n;
use crate::view::delete_dialog::shown;
use crate::view::style::danger;

const NAME_WIDTH: f32 = 320.0;
const BAR_HEIGHT: f32 = 48.0;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Choice {
    index: usize,
    name: String,
}

impl std::fmt::Display for Choice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.name)
    }
}

pub fn bar(skin: Skin, chooser: &Chooser) -> Element<'_, Message> {
    let content = match &chooser.replace {
        Some(paths) => replacing(skin, paths),
        None => choosing(skin, chooser),
    };
    container(content)
        .padding([0.0, PADDING])
        .height(BAR_HEIGHT)
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}

fn choosing(skin: Skin, chooser: &Chooser) -> Element<'_, Message> {
    let mut line = row![].spacing(SPACING).align_y(Vertical::Center);
    if chooser.naming() {
        line = line.push(
            text_input(&i18n::text("chooser-name"), &chooser.name)
                .id(NAME_INPUT)
                .on_input(|name| Message::Chooser(ChooserEvent::Name(name)))
                .on_submit(Message::Chooser(ChooserEvent::Accept))
                .size(TEXT_SIZE)
                .padding(6)
                .width(NAME_WIDTH)
                .style(style::input),
        );
    }
    line = line.push(Space::new().width(Length::Fill));
    if let Some(filters) = filters(chooser) {
        line = line.push(filters);
    }
    line.push(action(
        skin,
        i18n::text("cancel"),
        ChooserEvent::Cancel,
        false,
    ))
    .push(action(
        skin,
        chooser.accept_label(),
        ChooserEvent::Accept,
        true,
    ))
    .into()
}

fn filters(chooser: &Chooser) -> Option<Element<'_, Message>> {
    if chooser.request.filters.len() < 2 {
        return None;
    }
    let options: Vec<Choice> = chooser
        .request
        .filters
        .iter()
        .enumerate()
        .map(|(index, filter)| Choice {
            index,
            name: filter.name.clone(),
        })
        .collect();
    let current = chooser.filter.and_then(|index| options.get(index).cloned());
    Some(
        pick_list(options, current, |choice| {
            Message::Chooser(ChooserEvent::Filter(choice.index))
        })
        .text_size(TEXT_SIZE)
        .into(),
    )
}

fn replacing(skin: Skin, paths: &[std::path::PathBuf]) -> Element<'static, Message> {
    let name = match paths {
        [path] => path
            .file_name()
            .map(|name| shown(&name.to_string_lossy()))
            .unwrap_or_default(),
        paths => paths.len().to_string(),
    };
    let question = match paths.len() {
        1 => i18n::with("chooser-replace-one", "name", name),
        _ => i18n::with("chooser-replace-many", "count", name),
    };
    row![
        text(question)
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .width(Length::Fill),
        action(skin, i18n::text("cancel"), ChooserEvent::KeepEditing, false),
        button(text(i18n::text("chooser-replace")).size(TEXT_SIZE))
            .padding([6, 14])
            .on_press(Message::Chooser(ChooserEvent::Replace))
            .style(move |_, status| danger(skin, status)),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center)
    .into()
}

fn action(skin: Skin, label: String, event: ChooserEvent, main: bool) -> Element<'static, Message> {
    let highlight = Highlight::when(main);
    button(text(label).size(TEXT_SIZE))
        .padding([6, 14])
        .on_press(Message::Chooser(event))
        .style(move |_, status| style::entry(skin, highlight, status))
        .into()
}
