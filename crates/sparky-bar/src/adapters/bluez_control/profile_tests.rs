use super::{agent::PATH, context::PairContext};
use crate::adapters::{prompts::Broker, test_bus::Bus};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zbus::zvariant::OwnedObjectPath;

const DEVICE: &str = "/org/bluez/hci0/dev_7E_4F_15_BF_43_37";

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error.BREDR")]
enum Bredr {
    #[zbus(error)]
    ZBus(zbus::Error),
    ProfileUnavailable(String),
}

#[derive(Default)]
struct Link {
    up: AtomicBool,
    dropped: AtomicBool,
}

struct Manager;
#[zbus::interface(name = "org.bluez.AgentManager1")]
impl Manager {
    fn register_agent(&self, _path: OwnedObjectPath, _capability: &str) {}
    fn unregister_agent(&self, _path: OwnedObjectPath) {}
}

struct Device(Arc<Link>);
#[zbus::interface(name = "org.bluez.Device1")]
impl Device {
    fn connect(&self) -> Result<(), Bredr> {
        self.0.up.store(true, Ordering::Relaxed);
        Err(Bredr::ProfileUnavailable(
            "No more profiles to connect to".to_owned(),
        ))
    }
    fn disconnect(&self) {
        self.0.up.store(false, Ordering::Relaxed);
        self.0.dropped.store(true, Ordering::Relaxed);
    }
    fn cancel_pairing(&self) {}
    #[zbus(property)]
    fn alias(&self) -> String {
        "S22 Ultra de Palamar".to_owned()
    }
}

#[tokio::test]
async fn sem_perfis_para_conectar_a_falha_e_clara_e_o_enlace_vazio_e_fechado() {
    let bus = Bus::start();
    let bluez = bus.connection().await;
    bluez.request_name("org.bluez").await.unwrap();
    let link = Arc::new(Link::default());
    bluez
        .object_server()
        .at("/org/bluez", Manager)
        .await
        .unwrap();
    bluez
        .object_server()
        .at(DEVICE, Device(link.clone()))
        .await
        .unwrap();
    let client = bus.connection().await;
    let (broker, _events) = Broker::new();
    let context = Arc::new(PairContext::new(broker));
    client
        .object_server()
        .at(PATH, super::agent::Agent(context.clone()))
        .await
        .unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(4),
        super::pairing::connect(&client, context, DEVICE.to_owned()),
    )
    .await
    .unwrap();
    let Err(error) = result else {
        panic!("conexão sem perfis não pode ser sucesso");
    };
    let message = error.to_string();
    assert!(message.contains("S22 Ultra de Palamar"), "{message}");
    assert!(message.contains("nenhum perfil"), "{message}");
    assert!(!message.contains("org.bluez"), "{message}");
    assert!(link.dropped.load(Ordering::Relaxed));
    assert!(!link.up.load(Ordering::Relaxed));
}

#[test]
fn outras_falhas_seguem_como_vieram() {
    let falha = zbus::Error::Failure("br-connection-aborted-by-local".to_owned());
    assert!(!super::profiles::unavailable(&falha));
}
