use super::prompts::Broker;
use crate::{
    domain::{
        panel::PanelKind,
        prompt::{Answer, PromptEvent, PromptKind},
    },
    ports::prompts::Prompts,
};
use iced::futures::StreamExt;

#[tokio::test]
async fn confirmacao_retorna_ao_solicitante_e_fecha_formulario() {
    let (broker, mut events) = Broker::new();
    let caller = broker.clone();
    let request = tokio::spawn(async move {
        caller
            .request(PromptKind::Confirm {
                device: "Fone".to_owned(),
                code: Some(123),
            })
            .await
    });
    let PromptEvent::Show(prompt) = events.next().await.unwrap() else {
        panic!("sem formulário")
    };
    broker.answer(prompt.id, Answer::Accept);
    assert_eq!(request.await.unwrap(), Answer::Accept);
    assert!(matches!(events.next().await, Some(PromptEvent::Dismiss(id)) if id == prompt.id));
}

#[tokio::test]
async fn cancelar_bluetooth_rejeita_somente_suas_solicitacoes() {
    let (broker, mut events) = Broker::new();
    let caller = broker.clone();
    let request = tokio::spawn(async move {
        caller
            .request(PromptKind::Pin {
                device: "Teclado".to_owned(),
                numeric: true,
            })
            .await
    });
    let PromptEvent::Show(prompt) = events.next().await.unwrap() else {
        panic!("sem formulário")
    };
    broker.cancel(PanelKind::Network);
    assert!(!request.is_finished());
    broker.cancel(PanelKind::Bluetooth);
    assert_eq!(request.await.unwrap(), Answer::Reject);
    broker.answer(prompt.id, Answer::Accept);
}
