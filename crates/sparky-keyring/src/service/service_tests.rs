use zbus::zvariant::{OwnedObjectPath, Value};

use crate::ports::fake::MemoryStore;
use crate::ports::prompter::Ask;
use crate::service::paths;
use crate::service::service_support::{COLECAO, ITEM, SERVICO, cena, erro};

#[tokio::test]
async fn abre_sessao_plain_e_recusa_outros_algoritmos() {
    let cena = cena(&MemoryStore::default(), &[]).await;
    assert!(cena.sessao().await.as_str().starts_with("/org/freedesktop/secrets/session/"));
    let dh = cena.chamar(paths::SERVICE, SERVICO, "OpenSession", &("dh-ietf1024-sha256_aes128_cbc_pkcs7", Value::from(vec![1u8]))).await;
    assert_eq!(erro(dh).as_deref(), Some("org.freedesktop.DBus.Error.NotSupported"));
}

#[tokio::test]
async fn sem_cofre_guardar_pede_desbloqueio_e_a_janela_cria_a_senha() {
    let cena = cena(&MemoryStore::default(), &["nova"]).await;
    let sessao = cena.sessao().await;
    let trancado = cena.guardar(paths::COLLECTION, &sessao, "Git", "ana", false).await;
    assert!(matches!(trancado, Err(zbus::Error::MethodError(nome, _, _)) if nome.as_str() == "org.freedesktop.Secret.Error.IsLocked"));
    assert_eq!(cena.destrancar().await, (false, vec![paths::collection()]));
    assert_eq!(cena.janela.asked(), vec![Ask::Create]);
    assert_eq!(cena.guardar(paths::COLLECTION, &sessao, "Git", "ana", false).await.ok(), Some(paths::item(1)));
}

#[tokio::test]
async fn guarda_busca_le_e_apaga_pelo_apelido_padrao() {
    let cena = cena(&MemoryStore::default(), &["nova"]).await;
    cena.destrancar().await;
    let sessao = cena.sessao().await;
    let Ok(item) = cena.guardar(paths::DEFAULT_ALIAS, &sessao, "Git da Ana", "ana", false).await else { panic!("não guardou") };
    assert_eq!(cena.buscar("ana").await, (vec![item.clone()], vec![]));
    let segredos = cena.segredos(std::slice::from_ref(&item), &sessao).await.ok();
    assert_eq!(segredos.and_then(|mapa| mapa.get(&item).map(|segredo| segredo.value.clone())), Some(b"token-ana".to_vec()));
    let rotulo = cena.proxy(item.as_str(), ITEM).await.get_property::<String>("Label").await.ok();
    assert_eq!(rotulo.as_deref(), Some("Git da Ana"));
    assert!(cena.chamar(item.as_str(), ITEM, "Delete", &()).await.is_ok());
    assert_eq!(cena.buscar("ana").await, (vec![], vec![]));
}

#[tokio::test]
async fn substituir_atualiza_o_mesmo_item() {
    let cena = cena(&MemoryStore::default(), &["nova"]).await;
    cena.destrancar().await;
    let sessao = cena.sessao().await;
    let primeiro = cena.guardar(paths::COLLECTION, &sessao, "a", "ana", true).await.ok();
    let segundo = cena.guardar(paths::COLLECTION, &sessao, "b", "ana", true).await.ok();
    assert_eq!(primeiro, segundo);
    let outro = cena.guardar(paths::COLLECTION, &sessao, "c", "ana", false).await.ok();
    assert_eq!(outro, Some(paths::item(2)));
}

#[tokio::test]
async fn trancar_esconde_os_segredos_e_a_busca_mostra_como_trancado() {
    let store = MemoryStore::default();
    let cena = cena(&store, &["certa", "errada", "certa"]).await;
    cena.destrancar().await;
    let sessao = cena.sessao().await;
    let Ok(item) = cena.guardar(paths::COLLECTION, &sessao, "Git", "ana", false).await else { panic!("não guardou") };
    assert!(cena.chamar(paths::SERVICE, SERVICO, "Lock", &(vec![paths::collection()],)).await.is_ok());
    assert_eq!(cena.buscar("ana").await, (vec![], vec![item.clone()]));
    assert_eq!(cena.segredos(std::slice::from_ref(&item), &sessao).await.map(|mapa| mapa.len()).ok(), Some(0));
    assert_eq!(cena.proxy(paths::COLLECTION, COLECAO).await.get_property::<bool>("Locked").await.ok(), Some(true));
    assert_eq!(cena.destrancar().await, (false, vec![paths::collection()]));
    assert_eq!(cena.janela.asked(), vec![Ask::Create, Ask::Unlock, Ask::Retry]);
    assert_eq!(cena.buscar("ana").await, (vec![item], vec![]));
}

#[tokio::test]
async fn fechar_a_janela_responde_dispensado() {
    let cena = cena(&MemoryStore::default(), &[]).await;
    assert_eq!(cena.destrancar().await, (true, Vec::<OwnedObjectPath>::new()));
    assert_eq!(cena.proxy(paths::COLLECTION, COLECAO).await.get_property::<bool>("Locked").await.ok(), Some(true));
}

#[tokio::test]
async fn apelidos_e_sessoes_desconhecidas() {
    let cena = cena(&MemoryStore::default(), &[]).await;
    let padrao = cena.chamar(paths::SERVICE, SERVICO, "ReadAlias", &("default",)).await;
    assert_eq!(padrao.and_then(|m| m.body().deserialize::<OwnedObjectPath>()).ok(), Some(paths::collection()));
    let sessao = cena.chamar(paths::SERVICE, SERVICO, "ReadAlias", &("session",)).await;
    assert_eq!(sessao.and_then(|m| m.body().deserialize::<OwnedObjectPath>()).ok(), Some(paths::none()));
    let estranha = cena.segredos(&[paths::item(1)], &paths::session(99)).await;
    assert!(matches!(estranha, Err(zbus::Error::MethodError(nome, _, _)) if nome.as_str() == "org.freedesktop.Secret.Error.NoSession"));
}

#[tokio::test]
async fn o_portal_entrega_um_segredo_fixo_por_aplicativo() {
    let cena = cena(&MemoryStore::default(), &["nova"]).await;
    let primeiro = cena.segredo_do_portal("org.exemplo.App").await;
    assert_eq!(primeiro.as_ref().map(Vec::len), Some(64));
    assert_eq!(cena.segredo_do_portal("org.exemplo.App").await, primeiro);
    assert_ne!(cena.segredo_do_portal("org.exemplo.Outro").await, primeiro);
    assert_eq!(cena.janela.asked(), vec![Ask::Create]);
}

#[tokio::test]
async fn o_portal_cancela_se_o_cofre_nao_for_destrancado() {
    let cena = cena(&MemoryStore::default(), &[]).await;
    assert_eq!(cena.segredo_do_portal("org.exemplo.App").await, None);
}
