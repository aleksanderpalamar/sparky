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
        return hint(
            skin,
            crate::i18n::with("network-manager-unavailable", "reason", &error.to_string()),
        );
    }
    if settings.network.profiles.is_empty() {
        return hint(skin, crate::i18n::text("network-no-connection"));
    }
    let Some(draft) = &settings.network.draft else {
        return hint(skin, crate::i18n::text("network-ipv4-unavailable"));
    };
    column![
        caption(skin, &crate::i18n::text("network-active")),
        profiles(settings),
        caption(skin, &crate::i18n::text("network-method")),
        methods(skin, draft),
        addressing(settings, draft),
        dns(skin, draft),
        hint(skin, crate::i18n::text("network-ipv6-note")),
        row![
            Space::new().width(Length::Fill),
            action(
                skin,
                crate::i18n::text("network-save"),
                Message::NetworkSave
            )
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
        ("network-auto", Method::Auto),
        ("network-manual", Method::Manual),
    ]
    .into_iter()
    .map(|(label, method)| {
        option_row(
            skin,
            crate::i18n::text(label),
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
            caption(skin, &crate::i18n::text("network-address")),
            input(
                skin,
                "network-ipv4-address",
                "network-example-address",
                Field::Address,
                &draft.address
            ),
            input(
                skin,
                "network-gateway",
                "network-example-gateway",
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
    body.push(caption(skin, &crate::i18n::text("network-dns-servers")))
        .push(input(
            skin,
            "network-primary-dns",
            "network-example-primary-dns",
            Field::PrimaryDns,
            &draft.primary_dns,
        ))
        .push(input(
            skin,
            "network-secondary-dns",
            "network-example-secondary-dns",
            Field::SecondaryDns,
            &draft.secondary_dns,
        ))
        .into()
}
