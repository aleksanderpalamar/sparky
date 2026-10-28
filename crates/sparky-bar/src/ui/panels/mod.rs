mod audio;
mod bluetooth;
mod forms;
mod network;
mod style;

use iced::widget::{button, column, container, row, rule, scrollable, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::{message::Message, panel_state::Panels};
use crate::domain::panel::PanelKind;

pub fn view<'a>(skin: &Skin, panels: &'a Panels, kind: PanelKind) -> Element<'a, Message> {
    let title = match kind {
        PanelKind::Audio => "Som",
        PanelKind::Network => "Rede",
        PanelKind::Bluetooth => "Bluetooth",
    };
    let header = row![
        text(title)
            .size(16)
            .color(skin.foreground)
            .width(Length::Fill),
        action(skin, "×", Message::ClosePanel)
    ]
    .spacing(8);
    let normal = match kind {
        PanelKind::Audio => audio::view(skin, panels),
        PanelKind::Network => network::view(skin, panels),
        PanelKind::Bluetooth => bluetooth::view(skin, panels),
    };
    let body = forms::prompt(skin, panels).unwrap_or(normal);
    let mut content = column![
        header,
        rule::horizontal(1),
        scrollable(body).height(Length::Fill)
    ]
    .spacing(12);
    if panels.pending.contains_key(&kind) {
        content = content.push(text("Operação em andamento…").size(12).color(skin.accent));
    }
    if let Some(error) = panels.errors.get(&kind) {
        content = content.push(text(error).size(12).color(skin.urgent));
    }
    let skin = *skin;
    container(
        content.push(
            text(match panels.input_focus {
                crate::app::panel_state::InputFocus::Yielded => "Aguarde ou autentique · × fecha",
                _ => "Esc fecha · clique no ícone alterna",
            })
            .size(11)
            .color(skin.dark_foreground),
        ),
    )
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
mod fixtures;
#[cfg(test)]
mod render_tests;
