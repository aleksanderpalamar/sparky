use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, SPACING, TEXT_SIZE};

use super::network_fields::{auto_dns, caption, choice, input, lease};
use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::network::{Draft, Field, Method};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if let Some(error) = &settings.network.error {
        return text(format!("NetworkManager indisponível: {error}"))
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .into();
    }
    if settings.network.profiles.is_empty() {
        return text("Nenhuma conexão de rede ativa para configurar")
            .size(TEXT_SIZE)
            .color(skin.dark_foreground)
            .into();
    }
    let Some(draft) = &settings.network.draft else {
        return text("Configuração IPv4 indisponível")
            .size(TEXT_SIZE)
            .color(skin.urgent)
            .into();
    };
    let body = column![
        caption(settings, "Conexão ativa"),
        profiles(settings),
        caption(settings, "Método IPv4"),
        methods(settings, draft),
        addressing(settings, draft),
        dns(settings, draft),
        caption(settings, "As configurações IPv6 permanecem como estão."),
        button(text("Salvar e aplicar").size(TEXT_SIZE))
            .on_press(Message::NetworkSave)
            .style(move |_, status| style::entry(skin, Highlight::Selected, status)),
    ]
    .spacing(SPACING);
    scrollable(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn profiles<'a>(settings: &'a Settings) -> Element<'a, Message> {
    let skin = settings.context.skin;
    settings
        .network
        .profiles
        .iter()
        .enumerate()
        .fold(column![].spacing(4), |body, (index, profile)| {
            let selected = index == settings.network.selected;
            body.push(
                button(text(format!("{} · {}", profile.name, profile.interface)).size(TEXT_SIZE))
                    .on_press(Message::NetworkSelect(index))
                    .style(move |_, status| style::entry(skin, Highlight::when(selected), status)),
            )
        })
        .into()
}

fn methods<'a>(settings: &Settings, draft: &Draft) -> Element<'a, Message> {
    row![
        choice(
            settings,
            "Automático (DHCP)",
            draft.method == Method::Auto,
            Message::NetworkMethod(Method::Auto),
        ),
        choice(
            settings,
            "Manual",
            draft.method == Method::Manual,
            Message::NetworkMethod(Method::Manual),
        ),
    ]
    .spacing(SPACING)
    .into()
}

fn addressing<'a>(settings: &Settings, draft: &'a Draft) -> Element<'a, Message> {
    match draft.method {
        Method::Auto => lease(
            settings,
            settings
                .network
                .profiles
                .get(settings.network.selected)
                .map(|profile| profile.lease.clone())
                .unwrap_or_default(),
        ),
        Method::Manual => column![
            input(
                settings,
                "Endereço IPv4 / prefixo",
                "ex.: 192.168.1.10/24",
                Field::Address,
                &draft.address,
            ),
            input(
                settings,
                "Gateway",
                "ex.: 192.168.1.1",
                Field::Gateway,
                &draft.gateway
            ),
        ]
        .spacing(SPACING)
        .into(),
    }
}

fn dns<'a>(settings: &Settings, draft: &'a Draft) -> Element<'a, Message> {
    let mut body = column![].spacing(SPACING);
    if draft.method == Method::Auto {
        body = body.push(auto_dns(settings, draft));
    }
    body.push(
        row![
            input(
                settings,
                "DNS primário",
                "ex.: 1.1.1.1",
                Field::PrimaryDns,
                &draft.primary_dns,
            ),
            input(
                settings,
                "DNS secundário",
                "ex.: 8.8.8.8",
                Field::SecondaryDns,
                &draft.secondary_dns,
            ),
        ]
        .spacing(SPACING),
    )
    .into()
}
