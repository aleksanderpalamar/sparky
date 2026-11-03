use crate::ports::fake::MemoryStore;
use crate::service::paths;
use crate::service::service_support::{ITEM, cena};
use crate::service::wire::WireSecret;

const UM_VALOR: &[u8] = b"\x08(oayays)\0";

#[tokio::test]
async fn get_secret_do_item_responde_um_unico_segredo_como_manda_a_especificacao() {
    let cena = cena(&MemoryStore::default(), &["nova"]).await;
    cena.destrancar().await;
    let sessao = cena.sessao().await;
    let Ok(item) = cena
        .guardar(paths::DEFAULT_ALIAS, &sessao, "gh", "ana", false)
        .await
    else {
        panic!("não guardou");
    };
    let Ok(resposta) = cena
        .chamar(item.as_str(), ITEM, "GetSecret", &(sessao.clone(),))
        .await
    else {
        panic!("sem resposta");
    };
    assert!(
        resposta
            .data()
            .windows(UM_VALOR.len())
            .any(|janela| janela == UM_VALOR)
    );
    let Ok((segredo,)) = resposta.body().deserialize::<(WireSecret,)>() else {
        panic!("o segredo deveria vir num único valor");
    };
    assert_eq!(segredo.value, b"token-ana".to_vec());
}
