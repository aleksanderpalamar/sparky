use crate::domain::item::{Attributes, NewItem};
use crate::domain::secret::Secret;
use crate::domain::vault::Existing;
use crate::keyring::{Found, Keyring, KeyringError, Status};
use crate::ports::fake::MemoryStore;
use crate::seal::{Kdf, SealError};

pub const RAPIDO: Kdf = Kdf { memory: 64, passes: 1, lanes: 1 };

fn git() -> Attributes {
    [("service".to_owned(), "git".to_owned())].into_iter().collect()
}

fn novo() -> NewItem {
    NewItem { label: "git".to_owned(), attributes: git(), secret: Secret::new(b"token".to_vec(), "text/plain") }
}

fn carregar(store: &MemoryStore) -> Keyring {
    let Ok(chaveiro) = Keyring::load(Box::new(store.clone()), RAPIDO) else { panic!("não carregou") };
    chaveiro
}

pub fn com_um_item(store: &MemoryStore) -> Keyring {
    let mut chaveiro = carregar(store);
    assert!(chaveiro.create(b"certa", 1).is_ok());
    assert!(chaveiro.change(|cofre| cofre.store(novo(), Existing::Keep, 2)).is_ok());
    chaveiro
}

#[test]
fn sem_arquivo_o_cofre_ainda_nao_existe() {
    let chaveiro = carregar(&MemoryStore::default());
    assert_eq!(chaveiro.status(), Status::Missing);
    assert!(chaveiro.is_locked());
    assert!(chaveiro.ids().is_empty());
}

#[test]
fn criar_destranca_e_grava_o_arquivo() {
    let store = MemoryStore::default();
    let mut chaveiro = carregar(&store);
    assert!(chaveiro.create(b"certa", 1).is_ok());
    assert_eq!(chaveiro.status(), Status::Unlocked);
    assert!(store.contents().is_some());
    assert!(matches!(chaveiro.create(b"outra", 1), Err(KeyringError::Exists)));
}

#[test]
fn ao_reabrir_o_cofre_fica_trancado_mas_a_busca_acha_os_itens() {
    let store = MemoryStore::default();
    com_um_item(&store);
    let chaveiro = carregar(&store);
    assert_eq!(chaveiro.status(), Status::Locked);
    assert_eq!(chaveiro.ids(), vec![1]);
    assert_eq!(chaveiro.search(&git()), Found { unlocked: vec![], locked: vec![1] });
    assert!(matches!(chaveiro.item(1), Err(KeyringError::Locked)));
}

#[test]
fn destrancar_exige_a_senha_certa() {
    let store = MemoryStore::default();
    com_um_item(&store);
    let mut chaveiro = carregar(&store);
    assert!(matches!(chaveiro.unlock(b"errada"), Err(KeyringError::Seal(SealError::WrongPassword))));
    assert!(chaveiro.unlock(b"certa").is_ok());
    assert_eq!(chaveiro.search(&git()), Found { unlocked: vec![1], locked: vec![] });
    assert_eq!(chaveiro.item(1).map(|item| item.secret.value.to_vec()).ok(), Some(b"token".to_vec()));
}

#[test]
fn trancar_esquece_o_conteudo() {
    let mut chaveiro = com_um_item(&MemoryStore::default());
    chaveiro.lock();
    assert_eq!(chaveiro.status(), Status::Locked);
    assert!(matches!(chaveiro.item(1), Err(KeyringError::Locked)));
    assert_eq!(chaveiro.ids(), vec![1]);
}

#[test]
fn mudar_com_o_cofre_trancado_ou_inexistente_e_recusado() {
    let mut chaveiro = carregar(&MemoryStore::default());
    assert!(matches!(chaveiro.change(|cofre| cofre.store(novo(), Existing::Keep, 1)), Err(KeyringError::Locked)));
    assert!(matches!(chaveiro.unlock(b"x"), Err(KeyringError::Missing)));
}

#[test]
fn se_a_gravacao_falha_o_cofre_na_memoria_nao_muda() {
    let store = MemoryStore::default();
    let mut chaveiro = carregar(&store.failing());
    assert!(matches!(chaveiro.create(b"x", 1), Err(KeyringError::Store(_))));
    assert_eq!(chaveiro.status(), Status::Missing);
    let mut chaveiro = com_um_item(&store);
    let antes = store.contents();
    let mut falhando = carregar(&store.failing());
    assert!(falhando.unlock(b"certa").is_ok());
    assert!(matches!(falhando.change(|cofre| cofre.delete(1, 3)), Err(KeyringError::Store(_))));
    assert_eq!(falhando.ids(), vec![1]);
    assert_eq!(store.contents(), antes);
    assert!(chaveiro.change(|cofre| cofre.delete(1, 3)).is_ok());
}

#[test]
fn arquivo_corrompido_impede_o_carregamento() {
    let store = MemoryStore::default();
    store.put("lixo");
    assert!(matches!(Keyring::load(Box::new(store), RAPIDO), Err(KeyringError::Seal(SealError::Corrupt))));
}
