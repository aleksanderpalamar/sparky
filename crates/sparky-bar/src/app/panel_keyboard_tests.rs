use super::{
    message::Message,
    panel_state::{InputFocus, NetworkClient},
    tray_test_support::{bar, messages},
    update::update,
};
use crate::{
    domain::{
        network::{Network, NetworkRequest},
        panel::PanelKind,
    },
    ports::network::NetworkControl,
};
use iced::futures::{
    future::BoxFuture,
    stream::{self, BoxStream},
};
use iced_layershell::reexport::KeyboardInteractivity;
use std::sync::Arc;

#[derive(Debug)]
struct FakeNetwork;
impl NetworkControl for FakeNetwork {
    fn send(&self, _request: NetworkRequest) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
    fn subscribe(&self) -> BoxStream<'static, Network> {
        Box::pin(stream::empty())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn painel_libera_teclado_para_polkit_e_recupera_apos_operacao() {
    let mut bar = bar();
    bar.panels.network_client = Some(NetworkClient(Arc::new(FakeNetwork)));
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Network)));
    let window = bar.panels.open.unwrap().window;
    let output = messages(update(
        &mut bar,
        Message::NetworkRequest(NetworkRequest::Wireless(true)),
    ));
    assert!(output.iter().any(|message| matches!(message, Message::KeyboardInteractivityChange { id, keyboard_interactivity: KeyboardInteractivity::None } if *id == window)));
    let _ = messages(update(&mut bar, Message::PanelUnfocused(window)));
    assert!(bar.panels.open.is_some());
    let completed = output
        .into_iter()
        .find(|message| matches!(message, Message::OperationFinished(..)))
        .unwrap();
    let output = messages(update(&mut bar, completed));
    assert!(output.iter().any(|message| matches!(message, Message::KeyboardInteractivityChange { id, keyboard_interactivity: KeyboardInteractivity::Exclusive } if *id == window)));
    let _ = messages(update(&mut bar, Message::PanelFocusSettled(window)));
    assert_eq!(bar.panels.input_focus, InputFocus::Interactive);
}
