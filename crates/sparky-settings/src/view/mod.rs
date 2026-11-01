pub mod about;
pub mod autostart;
pub mod fields;
pub mod kit;
pub mod media;
pub mod monitors;
pub mod network;
mod network_fields;
mod screens;
pub mod shortcuts;
pub mod status;
pub mod style;
pub mod system;
pub mod users;

use iced::widget::{Column, column, container, row, scrollable};
use iced::{Element, Length, Padding};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::section::Section;
use crate::view::kit::chrome::{TITLE, footer, header, separator};
use crate::view::kit::look::surface;
use crate::view::kit::nav::sidebar;
use crate::view::kit::reveal::CONTENT;
use crate::view::kit::rows::notice_bar;
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let body = row![sidebar(skin, settings.section), content(settings)].height(Length::Fill);
    let hints = status::hints(settings.section, settings.mode);
    container(column![
        header(skin),
        separator(skin),
        body,
        separator(skin),
        footer(skin, hints)
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| surface(skin.background))
    .into()
}

pub fn title(settings: &Settings) -> String {
    format!("{} — {TITLE}", settings.section.label())
}

fn content(settings: &Settings) -> Element<'_, Message> {
    let gutter = Padding {
        right: space::HUGE,
        ..Padding::ZERO
    };
    let scrolled = scrollable(
        container(section(settings))
            .width(Length::Fill)
            .padding(gutter),
    )
    .id(CONTENT)
    .height(Length::Fill);
    let mut body = Column::new().push(scrolled).spacing(space::XXL);
    if let Some(notice) = &settings.notice {
        body = body.push(notice_bar(settings.context.skin, notice));
    }
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(space::HUGE)
        .into()
}

fn section(settings: &Settings) -> Element<'_, Message> {
    match settings.section {
        Section::Appearance | Section::Keyboard | Section::Apps => fields::view(settings),
        Section::Autostart => autostart::view(settings),
        Section::Shortcuts => shortcuts::view(settings),
        Section::Monitors => monitors::view(settings),
        Section::Network => network::view(settings),
        Section::Media => media::view(settings),
        Section::Power | Section::DateTime | Section::Region | Section::Themes => {
            system::view(settings)
        }
        Section::About => about::view(settings),
        Section::Users => users::view(settings),
    }
}
