use std::time::Duration;

use tokio::sync::mpsc::{self, Receiver};
use zbus::Connection;

use crate::adapters::screensaver::{NAME, serve};
use crate::adapters::test_bus::Barramento;
use crate::app::Event;
use crate::domain::guard::Input;
use crate::domain::inhibitions::Hold;

async fn cena() -> (Barramento, Connection, Receiver<Event>) {
    let barramento = Barramento::iniciar();
    let daemon = barramento.conectar().await;
    let (sender, receiver) = mpsc::channel(8);
    serve(&daemon, sender)
        .await
        .unwrap_or_else(|error| panic!("{error}"));
    (barramento, daemon, receiver)
}

async fn pedir(
    cliente: &Connection,
    caminho: &str,
    metodo: &str,
    corpo: &(impl zbus::export::serde::Serialize + zbus::zvariant::DynamicType),
) -> zbus::Result<zbus::Message> {
    cliente
        .call_method(Some(NAME), caminho, Some(NAME), metodo, corpo)
        .await
}

async fn proximo(receiver: &mut Receiver<Event>) -> Option<Event> {
    tokio::time::timeout(Duration::from_secs(3), receiver.recv())
        .await
        .ok()
        .flatten()
}

#[tokio::test]
async fn o_pedido_de_tela_acesa_segura_e_o_cancelamento_solta() {
    let (barramento, _daemon, mut receiver) = cena().await;
    let cliente = barramento.conectar().await;
    let resposta = pedir(
        &cliente,
        "/org/freedesktop/ScreenSaver",
        "Inhibit",
        &("firefox", "vídeo"),
    )
    .await;
    let cookie: u32 = resposta
        .and_then(|mensagem| mensagem.body().deserialize())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_ne!(cookie, 0);
    assert_eq!(
        proximo(&mut receiver).await,
        Some(Event::Seen(Input::Idle(Hold::Held)))
    );
    assert!(
        pedir(
            &cliente,
            "/org/freedesktop/ScreenSaver",
            "UnInhibit",
            &(cookie,)
        )
        .await
        .is_ok()
    );
    assert_eq!(
        proximo(&mut receiver).await,
        Some(Event::Seen(Input::Idle(Hold::Free)))
    );
}

#[tokio::test]
async fn tambem_atende_no_caminho_curto() {
    let (barramento, _daemon, mut receiver) = cena().await;
    let cliente = barramento.conectar().await;
    assert!(
        pedir(&cliente, "/ScreenSaver", "Inhibit", &("vlc", "vídeo"))
            .await
            .is_ok()
    );
    assert_eq!(
        proximo(&mut receiver).await,
        Some(Event::Seen(Input::Idle(Hold::Held)))
    );
}

#[tokio::test]
async fn quem_cai_do_barramento_solta_o_pedido() {
    let (barramento, _daemon, mut receiver) = cena().await;
    let cliente = barramento.conectar().await;
    assert!(
        pedir(
            &cliente,
            "/org/freedesktop/ScreenSaver",
            "Inhibit",
            &("mpv", "vídeo")
        )
        .await
        .is_ok()
    );
    assert_eq!(
        proximo(&mut receiver).await,
        Some(Event::Seen(Input::Idle(Hold::Held)))
    );
    drop(cliente);
    assert_eq!(
        proximo(&mut receiver).await,
        Some(Event::Seen(Input::Idle(Hold::Free)))
    );
}
