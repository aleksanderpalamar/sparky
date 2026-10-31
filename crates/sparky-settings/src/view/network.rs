use iced::widget::{Column, Space, column, row};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::network_fields::{auto_dns, input, lease};
use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::network::{Draft, Field, Method};
use crate::view::kit::controls::{action, option_row};
use crate::view::kit::rows::{caption, hint};
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if let Some(error) = &settings.network.error {
        return hint(skin, format!("NetworkManager indisponível: {error}"));
    }
    if settings.network.profiles.is_empty() {
        return hint(skin, "Nenhuma conexão de rede ativa para configurar");
    }
    let Some(draft) = &settings.network.draft else {
        return hint(skin, "Configuração IPv4 indisponível");
    };
    column![
        caption(skin, "Conexão ativa"),
        profiles(settings),
        caption(skin, "Método IPv4"),
        methods(skin, draft),
        addressing(settings, draft),
        dns(skin, draft),
        hint(skin, "As configurações IPv6 permanecem como estão."),
        row![
            Space::new().width(Length::Fill),
            action(skin, "Salvar e aplicar", Message::NetworkSave)
        ],
    ]
    .spacing(space::MD)
    .into()
}

fn profiles(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let options = settings
        .network
        .profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| {
            let label = format!("{} · {}", profile.name, profile.interface);
            option_row(
                skin,
                label,
                index == settings.network.selected,
                Message::NetworkSelect(index),
            )
        });
    Column::with_children(options).spacing(space::XXS).into()
}

fn methods<'a>(skin: Skin, draft: &Draft) -> Element<'a, Message> {
    let options = [
        ("Automático (DHCP)", Method::Auto),
        ("Manual", Method::Manual),
    ]
    .into_iter()
    .map(|(label, method)| {
        option_row(
            skin,
            label,
            draft.method == method,
            Message::NetworkMethod(method),
        )
    });
    Column::with_children(options).spacing(space::XXS).into()
}

fn addressing<'a>(settings: &Settings, draft: &'a Draft) -> Element<'a, Message> {
    let skin = settings.context.skin;
    match draft.method {
        Method::Auto => lease(
            skin,
            settings
                .network
                .profiles
                .get(settings.network.selected)
                .map(|profile| profile.lease.clone())
                .unwrap_or_default(),
        ),
        Method::Manual => column![
            caption(skin, "Endereço"),
            input(
                skin,
                "Endereço IPv4 / prefixo",
                "ex.: 192.168.1.10/24",
                Field::Address,
                &draft.address
            ),
            input(
                skin,
                "Gateway",
                "ex.: 192.168.1.1",
                Field::Gateway,
                &draft.gateway
            ),
        ]
        .spacing(space::XXS)
        .into(),
    }
}

fn dns<'a>(skin: Skin, draft: &'a Draft) -> Element<'a, Message> {
    let mut body = column![].spacing(space::MD);
    if draft.method == Method::Auto {
        body = body.push(auto_dns(skin, draft));
    }
    body.push(caption(skin, "Servidores DNS"))
        .push(input(
            skin,
            "DNS primário",
            "ex.: 1.1.1.1",
            Field::PrimaryDns,
            &draft.primary_dns,
        ))
        .push(input(
            skin,
            "DNS secundário",
            "ex.: 8.8.8.8",
            Field::SecondaryDns,
            &draft.secondary_dns,
        ))
        .into()
}
