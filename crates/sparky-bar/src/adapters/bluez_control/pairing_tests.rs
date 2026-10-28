use super::{
    agent::{Agent, PATH},
    context::PairContext,
};
use crate::{
    adapters::{prompts::Broker, test_bus::Bus},
    domain::prompt::{Answer, PromptEvent},
    ports::prompts::Prompts,
};
use iced::futures::StreamExt;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use zbus::{Connection, Proxy, message::Header, zvariant::OwnedObjectPath};

struct State {
    connection: Connection,
    registration: Mutex<Option<(String, OwnedObjectPath)>>,
    paired: AtomicBool,
    trusted: AtomicBool,
    connected: AtomicBool,
}

struct Manager(Arc<State>);
#[zbus::interface(name = "org.bluez.AgentManager1")]
impl Manager {
    fn register_agent(
        &self,
        path: OwnedObjectPath,
        capability: &str,
        #[zbus(header)] header: Header<'_>,
    ) {
        assert_eq!(capability, "KeyboardDisplay");
        *self.0.registration.lock().unwrap() = Some((header.sender().unwrap().to_string(), path));
    }
    fn unregister_agent(&self, _path: OwnedObjectPath) {
        *self.0.registration.lock().unwrap() = None;
    }
}

struct Device(Arc<State>);
#[zbus::interface(name = "org.bluez.Device1")]
impl Device {
    async fn pair(&self) -> zbus::fdo::Result<()> {
        let (destination, path) = self.0.registration.lock().unwrap().clone().unwrap();
        let proxy = Proxy::new(&self.0.connection, destination, path, "org.bluez.Agent1").await?;
        proxy
            .call::<_, _, ()>(
                "RequestConfirmation",
                &(
                    OwnedObjectPath::try_from("/org/bluez/hci0/dev_1").unwrap(),
                    123456u32,
                ),
            )
            .await?;
        self.0.paired.store(true, Ordering::Relaxed);
        Ok(())
    }
    fn connect(&self) {
        assert!(self.0.registration.lock().unwrap().is_some());
        self.0.connected.store(true, Ordering::Relaxed);
    }
    fn cancel_pairing(&self) {}
    #[zbus(property)]
    fn trusted(&self) -> bool {
        self.0.trusted.load(Ordering::Relaxed)
    }
    #[zbus(property)]
    fn set_trusted(&self, trusted: bool) {
        self.0.trusted.store(trusted, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn pareamento_real_do_cliente_exibe_confirmacao_confia_e_conecta_antes_de_remover_agente() {
    let bus = Bus::start();
    let bluez = bus.connection().await;
    bluez.request_name("org.bluez").await.unwrap();
    let state = Arc::new(State {
        connection: bluez.clone(),
        registration: Mutex::default(),
        paired: AtomicBool::new(false),
        trusted: AtomicBool::new(false),
        connected: AtomicBool::new(false),
    });
    bluez
        .object_server()
        .at("/org/bluez", Manager(state.clone()))
        .await
        .unwrap();
    bluez
        .object_server()
        .at("/org/bluez/hci0/dev_1", Device(state.clone()))
        .await
        .unwrap();
    let client = bus.connection().await;
    let (broker, mut events) = Broker::new();
    let context = Arc::new(PairContext::new(broker.clone()));
    client
        .object_server()
        .at(PATH, Agent(context.clone()))
        .await
        .unwrap();
    let request = super::pairing::pair(
        &client,
        context,
        "/org/bluez/hci0/dev_1".to_owned(),
        "Fone".to_owned(),
    );
    let answer = async {
        let PromptEvent::Show(prompt) = events.next().await.unwrap() else {
            panic!("sem confirmação")
        };
        broker.answer(prompt.id, Answer::Accept);
    };
    let (result, _) = tokio::time::timeout(std::time::Duration::from_secs(4), async {
        tokio::join!(request, answer)
    })
    .await
    .unwrap();
    result.unwrap();
    assert!(state.paired.load(Ordering::Relaxed));
    assert!(state.trusted.load(Ordering::Relaxed));
    assert!(state.connected.load(Ordering::Relaxed));
    assert!(state.registration.lock().unwrap().is_none());
}
