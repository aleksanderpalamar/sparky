use iced::widget::{Row, button, column, container, row, scrollable, text};
use iced::{Element, Length, Task};
use sparky_ui::style::{self, Highlight};

use super::message::Message;
use super::workspace::{TAB_HEIGHT, Workspace};
use crate::domain::tabs::TabAction;

const SCROLLER: &str = "sparky-terminal-tabs";
const TAB_WIDTH: f32 = 180.0;

pub fn view(app: &Workspace) -> Element<'_, Message> {
    let content = match app.tabs.active() {
        Some(tab) => super::view::view(tab.id, &tab.value),
        None => container(text("Terminal")).height(Length::Fill).into(),
    };
    column![header(app), content].height(Length::Fill).into()
}

fn header(app: &Workspace) -> Element<'_, Message> {
    let skin = app.skin;
    let tabs = Row::with_children(app.tabs.iter().enumerate().map(|(index, tab)| {
        let id = tab.id;
        let highlight = Highlight::when(app.tabs.active_id() == Some(id));
        let label = caption(index, &tab.value.title);
        container(row![
            button(
                text(label)
                    .size(style::SMALL_TEXT)
                    .wrapping(text::Wrapping::None)
            )
            .clip(true)
            .width(Length::Fill)
            .height(TAB_HEIGHT)
            .on_press(Message::SelectTab(id))
            .style(move |_, status| style::entry(skin, highlight, status)),
            button(text("×").size(style::SMALL_TEXT))
                .height(TAB_HEIGHT)
                .on_press(Message::CloseTab(id))
                .style(move |_, status| style::entry(skin, highlight, status)),
        ])
        .width(TAB_WIDTH)
        .into()
    }));
    let strip = scrollable(tabs)
        .id(SCROLLER)
        .horizontal()
        .height(TAB_HEIGHT)
        .width(Length::Fill);
    let add = button(text("+").size(16))
        .width(TAB_HEIGHT)
        .height(TAB_HEIGHT)
        .on_press(Message::Tab(TabAction::New))
        .style(move |_, status| style::entry(skin, Highlight::Plain, status));
    container(row![strip, add])
        .height(TAB_HEIGHT)
        .width(Length::Fill)
        .style(move |_| style::panel(skin))
        .into()
}

pub fn reveal(app: &Workspace) -> Task<Message> {
    let index = app
        .tabs
        .iter()
        .position(|tab| Some(tab.id) == app.tabs.active_id())
        .unwrap_or(0);
    let last = app.tabs.iter().len().saturating_sub(1).max(1);
    iced::widget::operation::snap_to(
        SCROLLER,
        scrollable::RelativeOffset {
            x: index as f32 / last as f32,
            y: 0.0,
        },
    )
}

pub fn caption(index: usize, title: &str) -> String {
    let prefix = format!("{}: ", index + 1);
    let short: String = title
        .chars()
        .filter(|character| !character.is_control())
        .take(17_usize.saturating_sub(prefix.chars().count()))
        .collect();
    format!("{prefix}{short}")
}
