use super::{
    agent::{Agent, PATH},
    context::{PairContext, Target},
    discovery::Discoveries,
};
use crate::{
    adapters::{prompts::Broker, test_bus::Bus},
    domain::{
        panel::PanelKind,
        prompt::{Answer, PromptEvent},
    },
    ports::prompts::Prompts,
};
use iced::futures::StreamExt;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use zbus::{Connection, Proxy, message::Header, zvariant::ObjectPath};

#[derive(Default)]
struct Radio(Arc<Mutex<HashSet<String>>>);

#[zbus::interface(name = "org.bluez.Adapter1")]
impl Radio {
    fn start_discovery(&self, #[zbus(header)] header: Header<'_>) {
        self.0
            .lock()
            .unwrap()
            .insert(header.sender().unwrap().to_string());
    }
    fn stop_discovery(&self, #[zbus(header)] header: Header<'_>) {
        self.0
            .lock()
            .unwrap()
            .remove(header.sender().unwrap().as_str());
    }
}

#[tokio::test]
async fn descoberta_pertence_ao_cliente_e_nao_interrompe_outros_programas() {
    let bus = Bus::start();
    let bluez = bus.connection().await;
    bluez.request_name("org.bluez").await.unwrap();
    let owners = Arc::new(Mutex::new(HashSet::new()));
    bluez
        .object_server()
        .at("/org/bluez/hci0", Radio(owners.clone()))
        .await
        .unwrap();
    let external = bus.connection().await;
    let proxy = super::proxy(&external, "/org/bluez/hci0", "org.bluez.Adapter1")
        .await
        .unwrap();
    proxy.call::<_, _, ()>("StartDiscovery", &()).await.unwrap();
    let client = bus.connection().await;
    let discoveries = Arc::new(Discoveries::default());
    discoveries
        .stop(&client, "/org/bluez/hci0", None)
        .await
        .unwrap();
    assert_eq!(owners.lock().unwrap().len(), 1);
    discoveries
        .start(client.clone(), "/org/bluez/hci0".to_owned())
        .await
        .unwrap();
    discoveries
        .start(client.clone(), "/org/bluez/hci0".to_owned())
        .await
        .unwrap();
    assert_eq!(owners.lock().unwrap().len(), 2);
    discoveries
        .stop(&client, "/org/bluez/hci0", Some(99))
        .await
        .unwrap();
    assert_eq!(owners.lock().unwrap().len(), 2);
    discoveries.stop_all(&client).await;
    assert_eq!(owners.lock().unwrap().len(), 1);
}

async fn call<'a>(connection: &'a Connection, destination: &'a str) -> Proxy<'a> {
    Proxy::new(connection, destination, PATH, "org.bluez.Agent1")
        .await
        .unwrap()
}

#[tokio::test]
async fn agente_valida_remetente_e_dispositivo_e_confirma_codigo_pelo_painel() {
    let bus = Bus::start();
    let bluez = bus.connection().await;
    let server = bus.connection().await;
    let stranger = bus.connection().await;
    let (broker, mut events) = Broker::new();
    let context = Arc::new(PairContext::new(broker.clone()));
    *context.target.lock().unwrap() = Some(Target {
        owner: bluez.unique_name().unwrap().to_string(),
        device: "/org/bluez/hci0/dev_1".to_owned(),
        name: "Fone".to_owned(),
        canceled: false,
    });
    server
        .object_server()
        .at(PATH, Agent(context.clone()))
        .await
        .unwrap();
    let destination = server.unique_name().unwrap().as_str();
    let device = ObjectPath::try_from("/org/bluez/hci0/dev_1").unwrap();
    let rejected = call(&stranger, destination)
        .await
        .call::<_, _, ()>("RequestConfirmation", &(device.clone(), 123u32))
        .await;
    assert!(rejected.is_err());
    let wrong = ObjectPath::try_from("/org/bluez/hci0/another").unwrap();
    assert!(
        call(&bluez, destination)
            .await
            .call::<_, _, ()>("RequestConfirmation", &(wrong, 123u32))
            .await
            .is_err()
    );
    let proxy = call(&bluez, destination).await;
    let body = (device.clone(), 123u32);
    let request = proxy.call::<_, _, ()>("RequestConfirmation", &body);
    let answer = async {
        let PromptEvent::Show(prompt) = events.next().await.unwrap() else {
            panic!("sem pergunta")
        };
        broker.answer(prompt.id, Answer::Accept);
    };
    let (result, _) = tokio::join!(request, answer);
    result.unwrap();
    context.cancel();
    assert!(
        call(&bluez, destination)
            .await
            .call::<_, _, ()>("RequestAuthorization", &(device,))
            .await
            .is_err()
    );
    broker.cancel(PanelKind::Bluetooth);
}

#[test]
fn codigo_numerico_preserva_zeros_e_rejeita_sinais_ou_excesso() {
    assert_eq!(super::agent::parse_passkey("000042"), Some(42));
    for invalid in ["", "+1", "-1", "1234567", " 123", "abc"] {
        assert!(super::agent::parse_passkey(invalid).is_none());
    }
}
