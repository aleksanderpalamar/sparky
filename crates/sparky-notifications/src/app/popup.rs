use super::Message;
use crate::{
    client::Command,
    domain::Card,
    presentation::{Density, card, style},
};
use iced::{
    Element, Length,
    widget::{column, container, scrollable},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use sparky_theme::Skin;

#[derive(Debug, Clone)]
pub struct Geometry {
    pub width: u32,
    pub height: u32,
    pub output: Option<String>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            output: None,
        }
    }
}

pub fn geometry() -> Geometry {
    let Ok(mut client) = sparky_ipc::Client::from_env() else {
        return Geometry::default();
    };
    let Ok(sparky_ipc::Reply::Outputs { outputs }) = client.request(&sparky_ipc::Request::Outputs)
    else {
        return Geometry::default();
    };
    outputs
        .iter()
        .find(|output| output.focused)
        .or_else(|| outputs.first())
        .map_or_else(Geometry::default, |output| Geometry {
            width: output.width.max(1) as u32,
            height: output.height.max(1) as u32,
            output: Some(output.name.clone()),
        })
}

pub fn size(count: usize, geometry: &Geometry) -> (u32, u32) {
    (
        360.min(geometry.width.saturating_sub(20)).max(1),
        (count.min(3) as u32 * 200)
            .min(geometry.height.saturating_sub(58))
            .max(1),
    )
}

pub fn settings(size: (u32, u32), output: Option<String>) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some(size),
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Right,
        exclusive_zone: None,
        margin: Some((32, 10, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::None,
        output_option: output.map_or(OutputOption::Active, OutputOption::OutputName),
        events_transparent: false,
        namespace: Some("sparky-notifications".to_owned()),
    }
}

pub fn view<'a>(skin: &Skin, cards: &'a [Card]) -> Element<'a, Message> {
    let content = column(cards.iter().rev().map(|item| {
        card(
            skin,
            item,
            Density::Popup,
            |id, key| Message::Command(Command::Invoke { id, key }),
            Message::Command(Command::Dismiss(item.id)),
        )
    }))
    .spacing(8);
    let skin = *skin;
    container(scrollable(content).height(Length::Fill))
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| style::surface(skin))
        .into()
}
