use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use zbus::zvariant::{ObjectPath, OwnedValue};

use super::service::{FileChooser, PATH};
use crate::adapters::test_bus::Bus;
use crate::domain::choice::{Outcome, Request};
use crate::ports::chooser::{Closer, Dialogs, Shown};

const HANDLE: &str = "/org/freedesktop/portal/desktop/request/1_1/t";

struct Janela(Shown);

impl Dialogs for Janela {
    fn show(&self, _request: &Request, _closer: &Closer) -> Shown {
        self.0.clone()
    }
}

struct Espera;

impl Dialogs for Espera {
    fn show(&self, _request: &Request, closer: &Closer) -> Shown {
        for _ in 0..1000 {
            if closer.closed() {
                return Shown::Cancelled;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Shown::Failed
    }
}

type Resposta = (u32, HashMap<String, OwnedValue>);

async fn abrir(bus: &Bus, dialogs: Arc<dyn Dialogs>) -> (zbus::Connection, zbus::Connection) {
    let servidor = bus.connection().await;
    servidor
        .object_server()
        .at(PATH, FileChooser::new(dialogs))
        .await
        .unwrap();
    (servidor, bus.connection().await)
}

async fn pedir(cliente: &zbus::Connection, servidor: &zbus::Connection) -> Resposta {
    let destino = servidor.unique_name().unwrap().to_owned();
    let opcoes: HashMap<String, OwnedValue> = HashMap::new();
    let resposta = cliente
        .call_method(
            Some(destino),
            PATH,
            Some("org.freedesktop.impl.portal.FileChooser"),
            "OpenFile",
            &(
                ObjectPath::try_from(HANDLE).unwrap(),
                "app",
                "",
                "Enviar",
                opcoes,
            ),
        )
        .await
        .unwrap();
    resposta.body().deserialize().unwrap()
}

#[tokio::test]
async fn escolha_feita_responde_sucesso_com_as_uris() {
    let bus = Bus::start();
    let escolhido = Shown::Chosen(Outcome {
        paths: vec![PathBuf::from("/tmp/demo.mp4")],
        filter: None,
    });
    let (servidor, cliente) = abrir(&bus, Arc::new(Janela(escolhido))).await;
    let (codigo, resultados) = pedir(&cliente, &servidor).await;
    assert_eq!(codigo, 0);
    let uris = Vec::<String>::try_from(resultados["uris"].try_clone().unwrap()).unwrap();
    assert_eq!(uris, vec!["file:///tmp/demo.mp4".to_owned()]);
}

#[tokio::test]
async fn cancelar_ou_falhar_responde_sem_resultados() {
    for (shown, esperado) in [(Shown::Cancelled, 1), (Shown::Failed, 2)] {
        let bus = Bus::start();
        let (servidor, cliente) = abrir(&bus, Arc::new(Janela(shown))).await;
        let (codigo, resultados) = pedir(&cliente, &servidor).await;
        assert_eq!((codigo, resultados.is_empty()), (esperado, true));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn o_aplicativo_pode_fechar_o_pedido_e_a_janela_some() {
    let bus = Bus::start();
    let (servidor, cliente) = abrir(&bus, Arc::new(Espera)).await;
    let destino = servidor.unique_name().unwrap().to_owned();
    let fechar = cliente.clone();
    tokio::spawn(async move {
        for _ in 0..500 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            let fechado = fechar
                .call_method(
                    Some(destino.clone()),
                    HANDLE,
                    Some("org.freedesktop.impl.portal.Request"),
                    "Close",
                    &(),
                )
                .await;
            if fechado.is_ok() {
                return;
            }
        }
    });
    let (codigo, _) = pedir(&cliente, &servidor).await;
    assert_eq!(codigo, 1);
}
