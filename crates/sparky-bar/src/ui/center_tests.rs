use super::{
    center_test_support::{exercise, save},
    clock::clock,
    panels,
};
use crate::{
    adapters::test_bus::Bus,
    app::{center_state::CenterState, message::Message, panel_state::Panels},
    domain::panel::PanelKind,
};
use iced::{Point, Size, mouse};
use sparky_notifications::{
    client::{Client, Command},
    domain::{Center, Incoming},
};
use sparky_theme::{Palette, Skin};
use std::sync::Arc;

#[test]
fn clique_esquerdo_abre_central_e_direito_preserva_data_hora() {
    let skin = Skin::default();
    let size = Size::new(200.0, 26.0);
    for (button, left) in [(mouse::Button::Left, true), (mouse::Button::Right, false)] {
        let view = clock(
            &skin,
            "terça 12:34".to_owned(),
            Message::ToggleClock,
            Message::Panel(PanelKind::Clock),
        );
        let (_, messages) = exercise(
            iced::widget::container(view)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into(),
            skin,
            size,
            Point::new(10.0, 10.0),
            button,
        );
        assert!(match messages.as_slice() {
            [Message::Panel(PanelKind::Clock)] => left,
            [Message::ToggleClock] => !left,
            _ => false,
        });
    }
}

#[tokio::test]
async fn central_com_historico_renderiza_temas_e_mantem_dnd_clicavel_fora_da_rolagem() {
    let bus = Bus::start();
    let mut center = CenterState {
        client: Some(Arc::new(Client::on(bus.connection().await))),
        error: None,
        ..Default::default()
    };
    let mut domain = Center::default();
    for index in 0..30 {
        domain
            .notify(
                Incoming {
                    app: "Aplicativo".to_owned(),
                    title: format!("Notificação {index}"),
                    body: "Mensagem de teste para a central nativa do Sparky".repeat(4),
                    timeout: 5000,
                    ..Default::default()
                },
                index,
            )
            .unwrap();
    }
    center.snapshot = domain.snapshot();
    for (name, colors) in [
        (
            "escuro",
            "background = \"#17232d\"\nforeground = \"#d0d8df\"",
        ),
        (
            "claro",
            "background = \"#efefef\"\nforeground = \"#222222\"",
        ),
    ] {
        let skin = Skin::new(Palette::from_colors(colors));
        for (width, height) in [(760, 520), (620, 422)] {
            let size = Size::new(width as f32, height as f32);
            let panels = Panels::default();
            let view = panels::view(&skin, &panels, &center, PanelKind::Clock);
            let (pixels, messages) = exercise(
                view,
                skin,
                size,
                Point::new(60.0, height as f32 - 77.0),
                mouse::Button::Left,
            );
            assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
            save(&format!("central-{name}-{width}"), &pixels, size);
            assert!(
                messages.iter().any(|message| matches!(
                    message,
                    Message::NotificationCommand(Command::Dnd(true))
                )),
                "{messages:?}"
            );
        }
    }
}
