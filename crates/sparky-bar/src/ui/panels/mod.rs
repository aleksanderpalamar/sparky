mod audio;
mod bluetooth;
mod calendar;
mod center;
mod forms;
mod network;
mod network_metrics;
mod parts;
mod style;

use iced::widget::{button, column, container, rule, scrollable, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::{message::Message, panel_state::Panels};
use crate::domain::panel::PanelKind;

pub fn view<'a>(
    skin: &Skin,
    panels: &'a Panels,
    center: &'a crate::app::center_state::CenterState,
    kind: PanelKind,
) -> Element<'a, Message> {
    let normal = match kind {
        PanelKind::Clock => center::view(skin, center),
        PanelKind::Audio => audio::view(skin, panels),
        PanelKind::Network => network::view(skin, panels),
        PanelKind::Bluetooth => bluetooth::view(skin, panels),
    };
    let body = forms::prompt(skin, panels).unwrap_or(normal);
    let body = match kind {
        PanelKind::Clock => body,
        _ => scrollable(body).height(Length::Fill).into(),
    };
    let mut content = if kind == PanelKind::Clock {
        column![
            text("Calendário e notificações")
                .size(16)
                .color(skin.foreground),
            rule::horizontal(1),
            body
        ]
        .spacing(12)
    } else {
        column![body].spacing(10)
    };
    if panels.pending.contains_key(&kind) {
        content = content.push(text("Operação em andamento…").size(12).color(skin.accent));
    }
    if let Some(error) = panels.errors.get(&kind) {
        content = content.push(text(error).size(12).color(skin.urgent));
    }
    let skin = *skin;
    if kind == PanelKind::Clock {
        content = content.push(
            text("Esc fecha · clique no relógio alterna")
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    container(content)
        .padding(14)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| style::surface(skin))
        .into()
}

pub fn action<'a>(skin: &Skin, label: impl Into<String>, message: Message) -> Element<'a, Message> {
    let skin = *skin;
    button(text(label.into()).size(12))
        .padding([6, 8])
        .on_press(message)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(skin.fill(if hovered { 0.16 } else { 0.08 }).into()),
                text_color: skin.foreground,
                ..Default::default()
            }
        })
        .into()
}

#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod render_tests;
