use std::fmt;

use iced::widget::{column, pick_list, text};
use iced::{Element, Length};
use sparky_config::ResourceMode;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::performance::{Knob, MODES, mode_key};
use crate::view::kit::card::group;
use crate::view::kit::inputs::switch;
use crate::view::kit::look::{dropdown, menu};
use crate::view::kit::rows::Line;
use crate::view::kit::tokens::{size, space};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Choice(ResourceMode);

impl fmt::Display for Choice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = format!("performance-mode-{}", mode_key(self.0));
        formatter.write_str(&crate::i18n::text(&key))
    }
}

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let resources = &settings.config.resources;
    let enabled = Line::new(
        crate::i18n::text("performance-enabled"),
        switch(
            skin,
            resources.enabled,
            Message::PerformanceToggle(Knob::Enabled),
        ),
    )
    .note(crate::i18n::text("performance-enabled-note"))
    .view(skin);
    group(skin, vec![enabled, mode(settings)])
}

fn mode(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let current = settings.config.resources.mode;
    let options: Vec<Choice> = MODES.into_iter().map(Choice).collect();
    let picker = pick_list(options, Some(Choice(current)), |Choice(mode)| {
        Message::PerformanceMode(mode)
    })
    .text_size(size::BODY)
    .padding([space::MD, space::XL])
    .width(Length::Fill)
    .style(dropdown(skin))
    .menu_style(menu(skin));
    let note = format!("performance-mode-{}-note", mode_key(current));
    column![
        text(crate::i18n::text("performance-operation"))
            .size(size::SMALL)
            .color(skin.dark_foreground),
        picker,
        text(crate::i18n::text(&note))
            .size(size::SMALL)
            .color(skin.dark_foreground),
    ]
    .spacing(space::MD)
    .padding([space::LG, space::LG])
    .into()
}
