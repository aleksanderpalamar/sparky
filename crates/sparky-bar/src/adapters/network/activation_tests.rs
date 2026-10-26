use super::{ROOT, profiles::Settings};
use crate::{
    adapters::{prompts::Broker, test_bus::Bus},
    app::panel_flow_tests::wifi,
    domain::{network::NetworkRequest, secret::Secret},
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

type Captured = Arc<Mutex<Vec<(Settings, String)>>>;
struct Root(Captured);
#[zbus::interface(name = "org.freedesktop.NetworkManager")]
impl Root {
    fn add_and_activate_connection2(
        &self,
        settings: Settings,
        _device: OwnedObjectPath,
        _ap: OwnedObjectPath,
        options: HashMap<String, OwnedValue>,
    ) -> (
        OwnedObjectPath,
        OwnedObjectPath,
        HashMap<String, OwnedValue>,
    ) {
        let persist = <&str>::try_from(&options["persist"]).unwrap().to_owned();
        self.0.lock().unwrap().push((settings, persist));
        (
            "/profile/1".try_into().unwrap(),
            "/active/1".try_into().unwrap(),
            HashMap::new(),
        )
    }
}

struct Active(Arc<AtomicU32>);
#[zbus::interface(name = "org.freedesktop.NetworkManager.Connection.Active")]
impl Active {
    #[zbus(property)]
    fn state(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }
}

struct Manager;
#[zbus::interface(name = "org.freedesktop.NetworkManager.AgentManager")]
impl Manager {
    fn register(&self, _identifier: &str) {}
    fn unregister(&self) {}
}

#[tokio::test]
async fn conexao_nova_aguarda_estado_ativo_e_respeita_opcao_de_persistencia() {
    let bus = Bus::start();
    let nm = bus.connection().await;
    nm.request_name(super::SERVICE).await.unwrap();
    let captured = Captured::default();
    let state = Arc::new(AtomicU32::new(2));
    nm.object_server()
        .at(ROOT, Root(captured.clone()))
        .await
        .unwrap();
    nm.object_server()
        .at("/active/1", Active(state.clone()))
        .await
        .unwrap();
    nm.object_server()
        .at("/org/freedesktop/NetworkManager/AgentManager", Manager)
        .await
        .unwrap();
    let (prompts, _events) = Broker::new();
    for remember in [false, true] {
        let client = bus.connection().await;
        super::commands::send_on(
            &client,
            NetworkRequest::Join {
                network: wifi(),
                password: Secret::new("12345678".to_owned()),
                remember,
            },
            prompts.clone(),
        )
        .await
        .unwrap();
    }
    {
        let captured = captured.lock().unwrap();
        assert_eq!(captured[0].1, "volatile");
        assert_eq!(captured[1].1, "disk");
    }
    state.store(4, Ordering::Relaxed);
    let client = bus.connection().await;
    assert!(
        super::commands::send_on(
            &client,
            NetworkRequest::Join {
                network: wifi(),
                password: Secret::new("12345678".to_owned()),
                remember: false
            },
            prompts
        )
        .await
        .is_err()
    );
}
