pub mod about;
pub mod autostart;
pub mod fields;
pub mod monitors;
pub mod shortcuts;
pub mod status;
pub mod style;
pub mod system;

use iced::widget::{column, container, row, text};
use iced::{Element, Length};
use sparky_ui::sidebar::{Entry, sidebar};
use sparky_ui::style::{Highlight, PADDING, SMALL_TEXT, SPACING, TITLE_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::section::Section;
use crate::view::style::SECTION_HEADER;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let main = column![header(settings), content(settings)]
        .spacing(SPACING)
        .padding(PADDING)
        .width(Length::Fill)
        .height(Length::Fill);
    let body = row![sections(settings), main].height(Length::Fill);
    container(column![body, status::status(settings)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin))
        .into()
}

pub fn title(settings: &Settings) -> String {
    format!("{} — Configurações", settings.section.label())
}

fn sections(settings: &Settings) -> Element<'_, Message> {
    let entries = Section::ALL
        .iter()
        .map(|section| Entry {
            label: section.label(),
            highlight: Highlight::when(*section == settings.section),
            message: Message::Section(*section),
        })
        .collect();
    sidebar(settings.context.skin, "Configurações", entries)
}

fn header(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let section = settings.section;
    container(column![
        text(section.label()).size(TITLE_SIZE).color(skin.bright_foreground),
        text(section.summary()).size(SMALL_TEXT).color(skin.dark_foreground),
    ])
    .height(SECTION_HEADER)
    .into()
}

fn content(settings: &Settings) -> Element<'_, Message> {
    match settings.section {
        Section::Appearance | Section::Keyboard | Section::Apps => fields::view(settings),
        Section::Autostart => autostart::view(settings),
        Section::Shortcuts => shortcuts::view(settings),
        Section::Monitors => monitors::view(settings),
        Section::Power | Section::DateTime | Section::Region => system::view(settings),
        Section::About => about::view(settings),
    }
}
