use crate::adapters::test_bus::Bus;
use std::sync::{Arc, Mutex};
use zbus::zvariant::OwnedObjectPath;

const ADAPTER: &str = "/org/bluez/hci0";
const IDENTIDADE: &str = "/org/bluez/hci0/dev_D8_68_A0_A9_E6_0F";
const ANTIGO: &str = "/org/bluez/hci0/dev_7E_4F_15_BF_43_37";

struct Adapter(Arc<Mutex<Vec<String>>>);
#[zbus::interface(name = "org.bluez.Adapter1")]
impl Adapter {
    fn remove_device(&self, device: OwnedObjectPath) {
        if let Ok(mut removed) = self.0.lock() {
            removed.push(device.to_string());
        }
    }
}

async fn bluez(removed: Arc<Mutex<Vec<String>>>) -> (Bus, zbus::Connection, zbus::Connection) {
    let bus = Bus::start();
    let bluez = bus.connection().await;
    bluez.request_name("org.bluez").await.unwrap();
    bluez
        .object_server()
        .at(ADAPTER, Adapter(removed))
        .await
        .unwrap();
    let client = bus.connection().await;
    (bus, bluez, client)
}

#[tokio::test]
async fn desparear_remove_cada_registro_do_aparelho_no_adaptador() {
    let removed = Arc::new(Mutex::new(Vec::new()));
    let (_bus, _bluez, client) = bluez(removed.clone()).await;
    let devices = vec![IDENTIDADE.to_owned(), ANTIGO.to_owned()];
    super::removal::remove(&client, ADAPTER, &devices)
        .await
        .unwrap();
    assert_eq!(*removed.lock().unwrap(), devices);
}

#[tokio::test]
async fn registro_fora_do_adaptador_e_recusado_sem_remover_nada() {
    let removed = Arc::new(Mutex::new(Vec::new()));
    let (_bus, _bluez, client) = bluez(removed.clone()).await;
    let devices = vec![
        IDENTIDADE.to_owned(),
        "/org/bluez/hci1/dev_D8_68_A0_A9_E6_0F".to_owned(),
    ];
    let result = super::removal::remove(&client, ADAPTER, &devices).await;
    assert!(result.is_err());
    assert!(removed.lock().unwrap().is_empty());
}
