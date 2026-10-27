use super::{PRIVATE_NAME, PRIVATE_PATH, PUBLIC_NAME, PUBLIC_PATH, Shared, test_bus::Bus};
use crate::{
    client::{Client, Command},
    domain::{Snapshot, Urgency},
};
use std::collections::HashMap;
use zbus::{Connection, Proxy, zvariant::OwnedValue};

async fn send(
    connection: &Connection,
    replaces: u32,
    title: &str,
    hints: HashMap<String, OwnedValue>,
) -> u32 {
    Proxy::new(connection, PUBLIC_NAME, PUBLIC_PATH, PUBLIC_NAME)
        .await
        .unwrap()
        .call(
            "Notify",
            &(
                "Teste",
                replaces,
                "",
                title,
                "Corpo",
                vec!["default", "Abrir"],
                hints,
                5000i32,
            ),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn api_padrao_e_central_compartilham_dnd_historico_ids_e_fechamento() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let application = bus.connection().await;
    let controls = Client::on(bus.connection().await);
    controls.send(Command::Dnd(true)).await.unwrap();
    let id = send(
        &application,
        0,
        "Mensagem crítica",
        HashMap::from([("urgency".to_owned(), 2u8.into())]),
    )
    .await;
    let snapshot = controls.snapshot().await.unwrap();
    assert!(snapshot.dnd);
    assert_eq!(snapshot.items[0].notification.urgency, Urgency::Critical);
    assert!(shared.read(|center| center.popups().is_empty()).unwrap());
    assert_eq!(
        send(&application, id, "Atualizada", HashMap::new()).await,
        id
    );
    assert_eq!(controls.snapshot().await.unwrap().items.len(), 1);
    controls.send(Command::Dnd(false)).await.unwrap();
    assert!(shared.read(|center| center.popups().is_empty()).unwrap());
    controls
        .send(Command::Invoke {
            id,
            key: "default".to_owned(),
        })
        .await
        .unwrap();
    assert!(controls.snapshot().await.unwrap().items.is_empty());
    assert!(
        controls
            .send(Command::Invoke {
                id,
                key: "default".to_owned()
            })
            .await
            .is_err()
    );
}

#[tokio::test]
async fn outro_dono_de_notificacoes_nao_e_substituido() {
    let bus = Bus::start();
    let mako = bus.connection().await;
    mako.request_name(PUBLIC_NAME).await.unwrap();
    let native = bus.connection().await;
    assert!(super::serve(&native, Shared::new()).await.is_err());
    let owner = zbus::fdo::DBusProxy::new(&native)
        .await
        .unwrap()
        .get_name_owner(PUBLIC_NAME.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(owner.as_str(), mako.unique_name().unwrap().as_str());
}

#[tokio::test]
async fn centro_aberto_nao_duplica_popup_e_perda_do_leitor_libera_sua_lease() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let application = bus.connection().await;
    let reader_connection = bus.connection().await;
    let owner = reader_connection.unique_name().unwrap().to_string();
    let reader = Client::on(reader_connection);
    reader.send(Command::CenterOpen(true)).await.unwrap();
    send(&application, 0, "No painel", HashMap::new()).await;
    assert!(shared.read(|center| center.popups().is_empty()).unwrap());
    shared.mutate(|center| center.owner_lost(&owner)).unwrap();
    send(&application, 0, "Novo aviso", HashMap::new()).await;
    assert_eq!(shared.read(|center| center.popups().len()).unwrap(), 1);
    let proxy = Proxy::new(&application, PRIVATE_NAME, PRIVATE_PATH, PRIVATE_NAME)
        .await
        .unwrap();
    let json: String = proxy.call("Snapshot", &()).await.unwrap();
    let snapshot: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(snapshot.items.len(), 2);
}

#[tokio::test]
async fn protocolo_anuncia_capacidades_e_emite_acao_e_fechamento_do_cliente() {
    use iced::futures::StreamExt;
    use std::time::Duration;
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let application = bus.connection().await;
    let proxy = Proxy::new(&application, PUBLIC_NAME, PUBLIC_PATH, PUBLIC_NAME)
        .await
        .unwrap();
    let capabilities: Vec<String> = proxy.call("GetCapabilities", &()).await.unwrap();
    assert!(capabilities.contains(&"actions".to_owned()));
    assert!(!capabilities.contains(&"persistence".to_owned()));
    assert!(!capabilities.contains(&"body-markup".to_owned()));
    let info: (String, String, String, String) =
        proxy.call("GetServerInformation", &()).await.unwrap();
    assert_eq!(info.0, "Sparky");
    assert_eq!(info.3, "1.3");
    let mut actions = proxy.receive_signal("ActionInvoked").await.unwrap();
    let mut closed = proxy.receive_signal("NotificationClosed").await.unwrap();
    let id = send(
        &application,
        0,
        "Residente",
        HashMap::from([("resident".to_owned(), true.into())]),
    )
    .await;
    let controls = Client::on(bus.connection().await);
    controls
        .send(Command::Invoke {
            id,
            key: "default".to_owned(),
        })
        .await
        .unwrap();
    let action = tokio::time::timeout(Duration::from_secs(2), actions.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        action.body().deserialize::<(u32, String)>().unwrap(),
        (id, "default".to_owned())
    );
    assert!(controls.snapshot().await.unwrap().items[0].active);
    let _: () = proxy.call("CloseNotification", &(id,)).await.unwrap();
    let closed = tokio::time::timeout(Duration::from_secs(2), closed.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(closed.body().deserialize::<(u32, u32)>().unwrap(), (id, 3));
    assert!(controls.snapshot().await.unwrap().items.is_empty());
}
