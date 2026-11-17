use iced::widget::{Column, button, column, row, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::{action, parts};
use crate::app::message::Message;
use crate::domain::bluetooth::{BluetoothRequest, Device};
use crate::i18n;

pub struct Rows<'a> {
    pub skin: Skin,
    pub adapter: &'a str,
    pub busy: bool,
    pub unpairing: Option<&'a str>,
}

impl Rows<'_> {
    pub fn push<'a>(
        &self,
        mut body: Column<'a, Message>,
        devices: &[&'a Device],
    ) -> Column<'a, Message> {
        for device in devices {
            body = body.push(self.device(device));
        }
        body
    }

    fn device<'a>(&self, device: &'a Device) -> Element<'a, Message> {
        if !device.paired {
            return self.choice(device, "pair-lower", pair(device));
        }
        if self.unpairing == Some(device.path.as_str()) {
            return self.confirm(device);
        }
        let main = match device.connected {
            true => self.choice(
                device,
                "disconnect-lower",
                BluetoothRequest::Disconnect(device.path.clone()),
            ),
            false => self.choice(
                device,
                "connect-lower",
                BluetoothRequest::Connect(device.path.clone()),
            ),
        };
        row![main, self.trash(device)]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
    }

    fn choice<'a>(
        &self,
        device: &'a Device,
        label: &str,
        request: BluetoothRequest,
    ) -> Element<'a, Message> {
        parts::choice(
            &self.skin,
            device.kind.icon(),
            device.name.clone(),
            Some(i18n::text(label)),
            device.connected,
            (!self.busy).then_some(Message::BluetoothRequest(request)),
        )
    }

    fn trash<'a>(&self, device: &Device) -> Element<'a, Message> {
        let skin = self.skin;
        let unpair = Message::BluetoothUnpair(Some(device.path.clone()));
        button(parts::icon(&skin, "user-trash-symbolic", 16.0))
            .padding([7, 6])
            .on_press_maybe((!self.busy).then_some(unpair))
            .style(move |_, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                button::Style {
                    background: hovered.then(|| skin.fill(0.16).into()),
                    text_color: skin.foreground,
                    ..Default::default()
                }
            })
            .into()
    }

    fn confirm<'a>(&self, device: &'a Device) -> Element<'a, Message> {
        let question = i18n::with("bluetooth-unpair-question", "name", &device.name);
        column![
            text(question)
                .size(12)
                .color(self.skin.foreground)
                .width(Length::Fill),
            row![
                action(
                    &self.skin,
                    i18n::text("unpair"),
                    Message::BluetoothRequest(device.unpair(self.adapter)),
                ),
                action(
                    &self.skin,
                    i18n::text("cancel"),
                    Message::BluetoothUnpair(None)
                ),
            ]
            .spacing(8),
        ]
        .spacing(8)
        .padding([7, 6])
        .into()
    }
}

fn pair(device: &Device) -> BluetoothRequest {
    BluetoothRequest::Pair {
        device: device.path.clone(),
        name: device.name.clone(),
    }
}
