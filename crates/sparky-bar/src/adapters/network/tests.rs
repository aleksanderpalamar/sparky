use super::{agent::PATH, profiles::Settings, profiles::create as settings};
use crate::{
    adapters::{prompts::Broker, test_bus::Bus},
    app::panel_flow_tests::wifi,
    domain::{
        network::Security,
        prompt::{Answer, PromptEvent},
        secret::Secret,
    },
    ports::prompts::Prompts,
};
use iced::futures::StreamExt;
use std::sync::{Arc, Mutex};
use zbus::{message::Header, zvariant::ObjectPath};

#[test]
fn novo_perfil_preserva_ssid_binario_e_armazena_senha_no_networkmanager() {
    let mut network = wifi();
    network.ssid = vec![0xff, 0x61];
    let settings = settings(&network, "12345678", false).unwrap();
    let wifi = &settings["802-11-wireless"];
    assert_eq!(
        Vec::<u8>::try_from(wifi["ssid"].try_clone().unwrap()).unwrap(),
        [0xff, 0x61]
    );
    assert!(!bool::try_from(&settings["connection"]["autoconnect"]).unwrap());
    let security = &settings["802-11-wireless-security"];
    assert_eq!(u32::try_from(&security["psk-flags"]).unwrap(), 0);
    assert_eq!(<&str>::try_from(&security["key-mgmt"]).unwrap(), "wpa-psk");
    assert_eq!(<&str>::try_from(&security["psk"]).unwrap(), "12345678");
    network.security = Security::Open;
    assert!(
        !super::profiles::create(&network, "", true)
            .unwrap()
            .contains_key("802-11-wireless-security")
    );
    network.security = Security::Sae;
    assert_eq!(
        <&str>::try_from(
            &super::profiles::create(&network, "senha", true).unwrap()["802-11-wireless-security"]
                ["key-mgmt"]
        )
        .unwrap(),
        "sae"
    );
}

#[derive(Default)]
struct Manager(Arc<Mutex<Vec<String>>>);
#[zbus::interface(name = "org.freedesktop.NetworkManager.AgentManager")]
impl Manager {
    fn register(&self, identifier: &str, #[zbus(header)] header: Header<'_>) {
        self.0
            .lock()
            .unwrap()
            .push(format!("{identifier}:{}", header.sender().unwrap()));
    }
    fn unregister(&self) {
        self.0.lock().unwrap().clear();
    }
}

#[tokio::test]
async fn agente_wifi_rejeita_outros_clientes_e_entrega_senha_apenas_ao_nm() {
    let bus = Bus::start();
    let nm = bus.connection().await;
    nm.request_name(super::SERVICE).await.unwrap();
    let server = bus.connection().await;
    let stranger = bus.connection().await;
    let (broker, mut events) = Broker::new();
    let registration = Arc::new(Mutex::new(Vec::new()));
    nm.object_server()
        .at(
            "/org/freedesktop/NetworkManager/AgentManager",
            Manager(registration.clone()),
        )
        .await
        .unwrap();
    let settings = settings(&wifi(), "12345678", true).unwrap();
    let guard = super::agent_guard::AgentGuard::register(
        &server,
        broker.clone(),
        &settings,
        Some("/profile/1".to_owned()),
    )
    .await
    .unwrap();
    assert_eq!(registration.lock().unwrap().len(), 1);
    let destination = server.unique_name().unwrap().as_str();
    let proxy = zbus::Proxy::new(
        &stranger,
        destination,
        PATH,
        "org.freedesktop.NetworkManager.SecretAgent",
    )
    .await
    .unwrap();
    assert!(
        proxy
            .call::<_, _, Settings>(
                "GetSecrets",
                &(
                    &settings,
                    ObjectPath::try_from("/profile/1").unwrap(),
                    "802-11-wireless-security",
                    Vec::<String>::new(),
                    1u32
                )
            )
            .await
            .is_err()
    );
    let proxy = zbus::Proxy::new(
        &nm,
        destination,
        PATH,
        "org.freedesktop.NetworkManager.SecretAgent",
    )
    .await
    .unwrap();
    let body = (
        &settings,
        ObjectPath::try_from("/profile/1").unwrap(),
        "802-11-wireless-security",
        Vec::<String>::new(),
        1u32,
    );
    let request = proxy.call::<_, _, Settings>("GetSecrets", &body);
    let answer = async {
        let PromptEvent::Show(prompt) = events.next().await.unwrap() else {
            panic!("sem senha")
        };
        broker.answer(
            prompt.id,
            Answer::Secret(Secret::new("nova-senha".to_owned())),
        );
    };
    let (result, _) = tokio::join!(request, answer);
    assert_eq!(
        <&str>::try_from(&result.unwrap()["802-11-wireless-security"]["psk"]).unwrap(),
        "nova-senha"
    );
    guard.unregister().await;
    assert!(registration.lock().unwrap().is_empty());
}
