use iced::Element;
use sparky_ipc::{Hdr, OutputInfo, Vrr};

use crate::app::message::Message;
use crate::app::pickers::ScreenField;
use crate::app::state::Settings;
use crate::domain::screens::{ScreenChange, last_enabled};
use crate::view::kit::controls::reading;
use crate::view::kit::inputs::{locked_switch, picker, switch};
use crate::view::kit::rows::Line;

pub fn choice<'a>(
    settings: &'a Settings,
    output: &OutputInfo,
    field: ScreenField,
    current: String,
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let Some(state) = settings.pickers.screen(&output.name, field) else {
        return reading(skin, current);
    };
    let name = output.name.clone();
    picker(skin, state, &current, move |value| {
        let change = match field {
            ScreenField::Mode => ScreenChange::Mode(value),
            ScreenField::Scale => ScreenChange::Scale(value),
            ScreenField::Rotation => ScreenChange::Rotation(value),
        };
        Message::Screen(name.clone(), change)
    })
}

pub fn active<'a>(
    settings: &Settings,
    output: &OutputInfo,
    outputs: &[OutputInfo],
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let only = output.enabled && last_enabled(outputs, &output.name);
    let (control, note) = match only {
        true => (
            locked_switch(skin, true),
            "É a única tela ligada; desligá-la deixaria você sem imagem.",
        ),
        false => (
            switch(
                skin,
                output.enabled,
                Message::Screen(output.name.clone(), ScreenChange::Enabled(!output.enabled)),
            ),
            "Desligar remove a tela do espaço de trabalho.",
        ),
    };
    Line::new("Ativa", control).note(note).view(skin)
}

pub fn high_dynamic_range<'a>(
    settings: &Settings,
    output: &OutputInfo,
) -> Option<Element<'a, Message>> {
    let on = match output.hdr {
        Hdr::Unsupported => return None,
        Hdr::On => true,
        Hdr::Off => false,
    };
    let skin = settings.context.skin;
    let change = Message::Screen(output.name.clone(), ScreenChange::Hdr(!on));
    Some(Line::new("HDR", switch(skin, on, change)).view(skin))
}

pub fn variable_rate<'a>(settings: &Settings, output: &OutputInfo) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let on = match output.vrr {
        Vrr::Unsupported => return reading(skin, "não suportada"),
        Vrr::On => true,
        Vrr::Off => false,
    };
    switch(
        skin,
        on,
        Message::Screen(output.name.clone(), ScreenChange::Vrr(!on)),
    )
}

pub fn yes_no(value: bool) -> &'static str {
    match value {
        true => "sim",
        false => "não",
    }
}
