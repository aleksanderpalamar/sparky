use std::sync::Arc;

use super::shared::Shared;
use super::test_support::{FakeRunner, Mode, Scene};
use crate::domain::Secret;

#[tokio::test]
async fn autoridade_pode_pedir_autenticacao_e_o_cookie_nao_sai_pela_resposta() {
    let scene = Scene::new(Mode::Accept).await;
    assert!(
        scene
            .begin(&scene.authority, "cookie-privado")
            .await
            .is_ok()
    );
    let calls = scene.runner.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].identities[0].uid, 0);
    assert_eq!(calls[0].action_id, "org.fixture.mount");
}

#[tokio::test]
async fn outro_cliente_nao_pode_abrir_nem_cancelar_janela_de_senha() {
    let scene = Scene::new(Mode::Accept).await;
    assert!(scene.begin(&scene.stranger, "invasor").await.is_err());
    assert!(scene.cancel(&scene.stranger, "invasor").await.is_err());
    assert!(scene.runner.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn troca_da_autoridade_revoga_o_remetente_antigo() {
    let scene = Scene::new(Mode::Accept).await;
    scene
        .authority
        .release_name(super::AUTHORITY)
        .await
        .unwrap();
    scene.stranger.request_name(super::AUTHORITY).await.unwrap();
    assert!(scene.begin(&scene.authority, "antiga").await.is_err());
    assert!(scene.begin(&scene.stranger, "nova").await.is_ok());
    assert_eq!(scene.runner.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn cancelamento_da_autoridade_termina_o_pedido_sem_autorizar() {
    let scene = Scene::new(Mode::Wait).await;
    let begin = scene.begin(&scene.authority, "cancelavel");
    let cancel = async {
        scene.runner.entered.notified().await;
        assert!(scene.cancel(&scene.authority, "cancelavel").await.is_ok());
    };
    let (result, ()) = tokio::join!(begin, cancel);
    let Err(zbus::Error::MethodError(name, _, _)) = result else {
        panic!("cancelar precisa retornar erro de cancelamento")
    };
    assert_eq!(name.as_str(), "org.freedesktop.PolicyKit1.Error.Cancelled");
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), scene.shared.idle())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn cookie_repetido_nao_abre_uma_segunda_janela() {
    let scene = Scene::new(Mode::Wait).await;
    let first = scene.begin(&scene.authority, "duplicado");
    let other = async {
        scene.runner.entered.notified().await;
        assert!(scene.begin(&scene.authority, "duplicado").await.is_err());
        scene.cancel(&scene.authority, "duplicado").await.unwrap();
    };
    let (result, ()) = tokio::join!(first, other);
    assert!(result.is_err());
    assert_eq!(scene.runner.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn fila_serializa_pedidos_e_cancelamento_alcanca_pedidos_aguardando() {
    let runner = Arc::new(FakeRunner {
        calls: std::sync::Mutex::default(),
        entered: Arc::new(tokio::sync::Notify::new()),
        mode: Mode::Accept,
    });
    let shared = Shared::new(runner);
    let first = shared.claim(&Secret::new("primeiro".to_owned())).unwrap();
    let mut next = shared.claim(&Secret::new("seguinte".to_owned())).unwrap();
    let lock = shared.gate.lock().await;
    assert!(shared.gate.try_lock().is_err());
    shared.cancel(&Secret::new("seguinte".to_owned()));
    next.cancellation.changed().await.unwrap();
    assert!(*next.cancellation.borrow());
    assert!(!*first.cancellation.borrow());
    drop(next);
    drop(first);
    drop(lock);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), shared.idle())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn fila_e_limitada_e_encerramento_cancela_todos_os_pedidos() {
    let runner = Arc::new(FakeRunner {
        calls: Default::default(),
        entered: Arc::new(tokio::sync::Notify::new()),
        mode: Mode::Accept,
    });
    let shared = Shared::new(runner);
    let mut pending: Vec<_> = (0..32)
        .map(|id| shared.claim(&Secret::new(id.to_string())).unwrap())
        .collect();
    assert!(shared.claim(&Secret::new("excesso".to_owned())).is_err());
    shared.cancel_all();
    for request in &mut pending {
        request.cancellation.changed().await.unwrap();
        assert!(*request.cancellation.borrow());
    }
    drop(pending);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), shared.idle())
            .await
            .is_ok()
    );
}
