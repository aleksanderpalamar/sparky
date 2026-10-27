use super::{PUBLIC_NAME, PUBLIC_PATH, Shared, test_bus::Bus};
use crate::{
    client::{Client, Command},
    domain::Incoming,
};
use iced::futures::StreamExt;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

async fn until(mut condition: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while !condition() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn timer_expira_sinaliza_id_invalido_e_nao_redesenha_quando_ocioso() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let application = bus.connection().await;
    let proxy = zbus::Proxy::new(&application, PUBLIC_NAME, PUBLIC_PATH, PUBLIC_NAME)
        .await
        .unwrap();
    let mut closed = proxy.receive_signal("NotificationClosed").await.unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let observed = counter.clone();
    let state = shared.clone();
    let worker = tokio::spawn(async move {
        super::pump(&server, state, move || {
            observed.fetch_add(1, Ordering::SeqCst);
            true
        })
        .await
    });
    until(|| counter.load(Ordering::SeqCst) > 0).await;
    let id = shared
        .mutate(|center| {
            center.notify(
                Incoming {
                    sender: application.unique_name().unwrap().to_string(),
                    timeout: 30,
                    ..Default::default()
                },
                shared.now(),
            )
        })
        .unwrap()
        .unwrap();
    let signal = tokio::time::timeout(Duration::from_secs(2), closed.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(signal.body().deserialize::<(u32, u32)>().unwrap(), (id, 1));
    assert!(!shared.snapshot().unwrap().items[0].active);
    tokio::time::sleep(Duration::from_millis(20)).await;
    let updates = counter.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(80)).await;
    assert_eq!(counter.load(Ordering::SeqCst), updates);
    worker.abort();
    let _ = worker.await;
}

#[tokio::test]
async fn fechar_conexao_da_barra_libera_lease_e_preserva_notificacoes_do_remetente() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    let state = shared.clone();
    let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let started = ready.clone();
    let worker = tokio::spawn(async move {
        super::pump(&server, state, move || {
            started.store(true, Ordering::SeqCst);
            true
        })
        .await
    });
    until(|| ready.load(Ordering::SeqCst)).await;
    let reader = Client::on(bus.connection().await);
    reader.send(Command::CenterOpen(true)).await.unwrap();
    let revision = shared.snapshot().unwrap().revision;
    drop(reader);
    until(|| shared.snapshot().unwrap().revision > revision).await;
    shared
        .mutate(|center| {
            center.notify(
                Incoming {
                    sender: ":1.99".to_owned(),
                    timeout: 5000,
                    ..Default::default()
                },
                shared.now(),
            )
        })
        .unwrap()
        .unwrap();
    assert_eq!(shared.read(|center| center.popups().len()).unwrap(), 1);
    shared.mutate(|center| center.owner_lost(":1.99")).unwrap();
    assert_eq!(shared.read(|center| center.popups().len()).unwrap(), 1);
    worker.abort();
    let _ = worker.await;
}

#[tokio::test]
async fn leases_concorrentes_aplicam_ultimo_estado_e_prontidao_confere_ambos_os_nomes() {
    let bus = Bus::start();
    let server = bus.connection().await;
    let proxy = zbus::fdo::DBusProxy::new(&server).await.unwrap();
    assert!(!super::readiness::owned(&proxy).await);
    let shared = Shared::new();
    super::serve(&server, shared.clone()).await.unwrap();
    assert!(super::readiness::owned(&proxy).await);
    let client = Client::on(bus.connection().await);
    client.desire_center(true);
    client.desire_center(false);
    let (a, b) = tokio::join!(client.sync_center(), client.sync_center());
    a.unwrap();
    b.unwrap();
    shared
        .mutate(|center| center.notify(Incoming::default(), shared.now()))
        .unwrap()
        .unwrap();
    assert_eq!(shared.read(|center| center.popups().len()).unwrap(), 1);
}
