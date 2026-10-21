use std::collections::HashMap;
use std::io::Read;
use std::os::fd::OwnedFd;
use std::sync::Arc;

use futures_util::StreamExt;
use zbus::proxy::{Builder, CacheProperties};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Str, Value};
use zbus::{Connection, Guid, Proxy};

use crate::keyring::Keyring;
use crate::ports::fake::{MemoryStore, ScriptedPrompter};
use crate::seal::Kdf;
use crate::service::paths;
use crate::service::serve;
use crate::service::shared::Shared;
use crate::service::wire::{ATTRIBUTES, LABEL, WireSecret};

const RAPIDO: Kdf = Kdf { memory: 64, passes: 1, lanes: 1 };
pub const SERVICO: &str = "org.freedesktop.Secret.Service";
pub const COLECAO: &str = "org.freedesktop.Secret.Collection";
pub const ITEM: &str = "org.freedesktop.Secret.Item";

pub struct Cena {
    pub cliente: Connection,
    _servidor: Connection,
    pub janela: Arc<ScriptedPrompter>,
}

pub async fn cena(store: &MemoryStore, respostas: &[&'static str]) -> Cena {
    let Ok(chaveiro) = Keyring::load(Box::new(store.clone()), RAPIDO) else { panic!("não carregou") };
    let janela = Arc::new(ScriptedPrompter::new(respostas));
    let shared = Arc::new(Shared::new(chaveiro, janela.clone()));
    let Ok((um, outro)) = tokio::net::UnixStream::pair() else { panic!("sem soquete") };
    let servidor = async { zbus::connection::Builder::unix_stream(um).server(Guid::generate())?.p2p().build().await };
    let cliente = zbus::connection::Builder::unix_stream(outro).p2p().build();
    let Ok((servidor, cliente)) = tokio::try_join!(servidor, cliente) else { panic!("sem conexão") };
    assert!(serve(&servidor, shared).await.is_ok());
    Cena { cliente, _servidor: servidor, janela }
}

pub fn erro(resultado: zbus::Result<zbus::Message>) -> Option<String> {
    match resultado {
        Err(zbus::Error::MethodError(nome, _, _)) => Some(nome.to_string()),
        _ => None,
    }
}

impl Cena {
    pub async fn proxy(&self, caminho: &str, interface: &str) -> Proxy<'static> {
        let construtor = Builder::<Proxy<'static>>::new(&self.cliente).cache_properties(CacheProperties::No);
        let pronto = construtor.destination("org.freedesktop.secrets").and_then(|b| b.path(caminho.to_owned())).and_then(|b| b.interface(interface.to_owned()));
        let feito = match pronto {
            Ok(construtor) => construtor.build().await,
            Err(erro) => Err(erro),
        };
        match feito {
            Ok(proxy) => proxy,
            Err(erro) => panic!("sem proxy: {erro}"),
        }
    }

    pub async fn chamar(&self, caminho: &str, interface: &str, metodo: &str, corpo: &(impl serde::Serialize + zbus::zvariant::DynamicType)) -> zbus::Result<zbus::Message> {
        self.proxy(caminho, interface).await.call_method(metodo, corpo).await
    }

    pub async fn sessao(&self) -> OwnedObjectPath {
        let resposta = self.chamar(paths::SERVICE, SERVICO, "OpenSession", &("plain", Value::from(""))).await;
        let Ok((_, caminho)) = resposta.and_then(|m| m.body().deserialize::<(OwnedValue, OwnedObjectPath)>()) else { panic!("sem sessão") };
        caminho
    }

    pub async fn destrancar(&self) -> (bool, Vec<OwnedObjectPath>) {
        let resposta = self.chamar(paths::SERVICE, SERVICO, "Unlock", &(vec![paths::collection()],)).await;
        let Ok((_, pedido)) = resposta.and_then(|m| m.body().deserialize::<(Vec<OwnedObjectPath>, OwnedObjectPath)>()) else { panic!("sem pedido") };
        let janela = self.proxy(pedido.as_str(), "org.freedesktop.Secret.Prompt").await;
        let Ok(mut sinais) = janela.receive_signal("Completed").await else { panic!("sem sinal") };
        assert!(janela.call_method("Prompt", &("",)).await.is_ok());
        let Some(sinal) = sinais.next().await else { panic!("o pedido não terminou") };
        let Ok((dispensado, resultado)) = sinal.body().deserialize::<(bool, OwnedValue)>() else { panic!("sinal estranho") };
        let Ok(objetos) = Vec::<OwnedObjectPath>::try_from(resultado) else { panic!("resultado estranho") };
        (dispensado, objetos)
    }

    pub async fn guardar(&self, caminho: &str, sessao: &OwnedObjectPath, rotulo: &str, usuario: &str, substituir: bool) -> zbus::Result<OwnedObjectPath> {
        let atributos: HashMap<String, String> = [("service", "git"), ("user", usuario)].iter().map(|(n, v)| (n.to_string(), v.to_string())).collect();
        let propriedades: HashMap<String, OwnedValue> =
            [(LABEL.to_owned(), OwnedValue::from(Str::from(rotulo.to_owned()))), (ATTRIBUTES.to_owned(), OwnedValue::from(atributos))].into_iter().collect();
        let segredo = WireSecret { session: sessao.clone(), parameters: Vec::new(), value: format!("token-{usuario}").into_bytes(), content_type: "text/plain".to_owned() };
        let resposta = self.chamar(caminho, COLECAO, "CreateItem", &(propriedades, segredo, substituir)).await?;
        let (item, _) = resposta.body().deserialize::<(OwnedObjectPath, OwnedObjectPath)>()?;
        Ok(item)
    }

    pub async fn buscar(&self, usuario: &str) -> (Vec<OwnedObjectPath>, Vec<OwnedObjectPath>) {
        let atributos: HashMap<&str, &str> = [("service", "git"), ("user", usuario)].into_iter().collect();
        let resposta = self.chamar(paths::SERVICE, SERVICO, "SearchItems", &(atributos,)).await;
        let Ok(achados) = resposta.and_then(|m| m.body().deserialize::<(Vec<OwnedObjectPath>, Vec<OwnedObjectPath>)>()) else { panic!("sem busca") };
        achados
    }

    pub async fn segredos(&self, itens: &[OwnedObjectPath], sessao: &OwnedObjectPath) -> zbus::Result<HashMap<OwnedObjectPath, WireSecret>> {
        let resposta = self.chamar(paths::SERVICE, SERVICO, "GetSecrets", &(itens.to_vec(), sessao.clone())).await?;
        resposta.body().deserialize()
    }

    pub async fn segredo_do_portal(&self, aplicativo: &str) -> Option<Vec<u8>> {
        let Ok((mut leitura, escrita)) = std::io::pipe() else { panic!("sem cano") };
        let canal = zbus::zvariant::OwnedFd::from(OwnedFd::from(escrita));
        let opcoes: HashMap<String, OwnedValue> = HashMap::new();
        let pedido = paths::prompt(999);
        let corpo = (pedido, aplicativo, canal, opcoes);
        let resposta = self.chamar("/org/freedesktop/portal/desktop", "org.freedesktop.impl.portal.Secret", "RetrieveSecret", &corpo).await;
        let Ok((codigo, _)) = resposta.and_then(|m| m.body().deserialize::<(u32, HashMap<String, OwnedValue>)>()) else { panic!("sem resposta") };
        if codigo != 0 {
            return None;
        }
        let mut segredo = vec![0u8; 64];
        leitura.read_exact(&mut segredo).ok()?;
        Some(segredo)
    }
}
