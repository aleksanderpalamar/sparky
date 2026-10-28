use super::{PUBLIC_NAME, PUBLIC_PATH, Shared, test_bus::Bus};
use crate::domain::{Error, Incoming};
use iced::futures::future::join_all;
use std::{collections::HashMap, time::Duration};
use zbus::{
    Connection, Proxy,
    zvariant::{OwnedValue, Value},
};

async fn notify(
    connection: &Connection,
    title: &str,
    hints: HashMap<String, OwnedValue>,
) -> zbus::Result<u32> {
    Proxy::new(connection, PUBLIC_NAME, PUBLIC_PATH, PUBLIC_NAME)
        .await?
        .call(
            "Notify",
            &(
                "Teste",
                0u32,
                "",
                title,
                "Corpo",
                Vec::<String>::new(),
                hints,
                5000i32,
            ),
        )
        .await
}

fn image() -> HashMap<String, OwnedValue> {
    let data = (1i32, 1i32, 4i32, true, 8i32, 4i32, vec![255u8, 0, 0, 255]);
    HashMap::from([(
        "image-data".to_owned(),
        Value::from(data).try_into().unwrap(),
    )])
}

#[tokio::test]
async fn rajada_preserva_texto_quando_os_dois_workers_de_imagem_estao_ocupados() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let occupied = shared.images.clone().acquire_many_owned(2).await.unwrap();
    let client = bus.connection().await;
    let requests = (0..8).map(|index| {
        let client = client.clone();
        async move {
            let hints = if index % 2 == 0 {
                HashMap::new()
            } else {
                image()
            };
            let title = format!("Mensagem {index}");
            notify(&client, &title, hints).await
        }
    });
    let results = tokio::time::timeout(Duration::from_secs(2), join_all(requests))
        .await
        .unwrap();
    let ids = results
        .into_iter()
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .unwrap();
    assert_eq!(ids.len(), 8);
    let snapshot = shared.snapshot().unwrap();
    assert_eq!(snapshot.items.len(), 8);
    for card in snapshot.items {
        assert!(ids.contains(&card.id));
        assert!(card.notification.title.starts_with("Mensagem "));
        assert_eq!(card.notification.body, "Corpo");
        assert!(card.notification.picture.is_none());
        assert!(card.notification.icon.is_empty());
    }
    assert_eq!(shared.read(|center| center.popups().len()).unwrap(), 3);
    assert_eq!(shared.images.available_permits(), 0);
    drop(occupied);
    assert_eq!(shared.images.available_permits(), 2);
    let id = notify(&client, "Imagem após a rajada", image())
        .await
        .unwrap();
    let snapshot = shared.snapshot().unwrap();
    let card = snapshot.items.iter().find(|card| card.id == id).unwrap();
    assert_eq!(
        card.notification.picture.as_ref().unwrap().rgba,
        [255, 0, 0, 255]
    );
    assert_eq!(shared.images.available_permits(), 2);
}

#[tokio::test]
async fn fallback_de_imagem_preserva_o_limite_de_notificacoes_ativas() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    for now in 0..128 {
        shared
            .mutate(|center| center.notify(Incoming::default(), now))
            .unwrap()
            .unwrap();
    }
    let _occupied = shared.images.clone().acquire_many_owned(2).await.unwrap();
    let client = bus.connection().await;
    let error = notify(&client, "Acima do limite", HashMap::new())
        .await
        .unwrap_err();
    assert!(error.to_string().contains(&Error::Full.to_string()));
    assert_eq!(shared.snapshot().unwrap().items.len(), 100);
}
