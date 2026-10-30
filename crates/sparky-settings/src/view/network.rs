use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::network::{Field, Method};

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
    let profiles = settings.network.profiles.iter().enumerate().fold(
        column![].spacing(4),
        |body, (index, profile)| {
            let selected = index == settings.network.selected;
            body.push(
                button(text(format!("{} · {}", profile.name, profile.interface)).size(TEXT_SIZE))
                    .on_press(Message::NetworkSelect(index))
                    .style(move |_, status| style::entry(skin, Highlight::when(selected), status)),
            )
        },
    );
    let auto = method(settings, "Automático (DHCP)", Method::Auto);
    let manual = method(settings, "Manual", Method::Manual);
    let mut body = column![
        text("Conexão ativa")
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
        profiles,
        text("Método IPv4")
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
        row![auto, manual].spacing(SPACING),
    ]
    .spacing(SPACING);
    if draft.method == Method::Manual {
        body = body.push(input(
            settings,
            "Endereço IPv4 / prefixo",
            "192.168.1.10/24",
            Field::Address,
            &draft.address,
        ));
        body = body.push(input(
            settings,
            "Gateway",
            "192.168.1.1",
            Field::Gateway,
            &draft.gateway,
        ));
    }
    body = body.push(input(
        settings,
        "Servidores DNS IPv4",
        "1.1.1.1, 8.8.8.8",
        Field::Dns,
        &draft.dns,
    ));
    body = body.push(
        text("As configurações IPv6 permanecem como estão.")
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    );
    body = body.push(
        button(text("Salvar e aplicar").size(TEXT_SIZE))
            .on_press(Message::NetworkSave)
            .style(move |_, status| style::entry(skin, Highlight::Selected, status)),
    );
    scrollable(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn method<'a>(settings: &'a Settings, label: &'static str, method: Method) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let selected = settings
        .network
        .draft
        .as_ref()
        .is_some_and(|draft| draft.method == method);
    button(text(label).size(TEXT_SIZE))
        .on_press(Message::NetworkMethod(method))
        .style(move |_, status| style::entry(skin, Highlight::when(selected), status))
        .into()
}

fn input<'a>(
    settings: &Settings,
    label: &'static str,
    placeholder: &'static str,
    field: Field,
    value: &'a str,
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    column![
        text(label).size(SMALL_TEXT).color(skin.dark_foreground),
        text_input(placeholder, value)
            .on_input(move |value| Message::NetworkInput(field, value))
            .on_submit(Message::NetworkSave)
            .size(TEXT_SIZE)
            .style(style::input),
    ]
    .spacing(4)
    .into()
}
