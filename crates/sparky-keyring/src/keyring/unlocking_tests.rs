use std::sync::Mutex;

use crate::keyring::keyring_tests::{RAPIDO, com_um_item};
use crate::keyring::unlocking::{Outcome, unlock};
use crate::keyring::{Keyring, Status};
use crate::ports::fake::{MemoryStore, ScriptedPrompter};
use crate::ports::prompter::Ask;

fn trancado() -> Mutex<Keyring> {
    let store = MemoryStore::default();
    com_um_item(&store);
    let Ok(chaveiro) = Keyring::load(Box::new(store), RAPIDO) else { panic!("não carregou") };
    Mutex::new(chaveiro)
}

fn status(chaveiro: &Mutex<Keyring>) -> Option<Status> {
    chaveiro.lock().ok().map(|chaveiro| chaveiro.status())
}

#[test]
fn sem_cofre_a_janela_pede_uma_senha_nova() {
    let Ok(chaveiro) = Keyring::load(Box::new(MemoryStore::default()), RAPIDO) else { panic!("não carregou") };
    let chaveiro = Mutex::new(chaveiro);
    let janela = ScriptedPrompter::new(&["nova"]);
    assert_eq!(unlock(&chaveiro, &janela, 5), Outcome::Unlocked);
    assert_eq!(janela.asked(), vec![Ask::Create]);
    assert_eq!(status(&chaveiro), Some(Status::Unlocked));
}

#[test]
fn senha_nova_vazia_e_pedida_de_novo() {
    let Ok(chaveiro) = Keyring::load(Box::new(MemoryStore::default()), RAPIDO) else { panic!("não carregou") };
    let chaveiro = Mutex::new(chaveiro);
    let janela = ScriptedPrompter::new(&["", "nova"]);
    assert_eq!(unlock(&chaveiro, &janela, 5), Outcome::Unlocked);
    assert_eq!(janela.asked(), vec![Ask::Create, Ask::Create]);
}

#[test]
fn senha_errada_pede_de_novo_avisando() {
    let chaveiro = trancado();
    let janela = ScriptedPrompter::new(&["errada", "certa"]);
    assert_eq!(unlock(&chaveiro, &janela, 5), Outcome::Unlocked);
    assert_eq!(janela.asked(), vec![Ask::Unlock, Ask::Retry]);
}

#[test]
fn cancelar_deixa_o_cofre_como_estava() {
    let chaveiro = trancado();
    let janela = ScriptedPrompter::new(&["errada"]);
    assert_eq!(unlock(&chaveiro, &janela, 5), Outcome::Dismissed);
    assert_eq!(janela.asked(), vec![Ask::Unlock, Ask::Retry]);
    assert_eq!(status(&chaveiro), Some(Status::Locked));
}

#[test]
fn ja_destrancado_nao_abre_janela() {
    let chaveiro = Mutex::new(com_um_item(&MemoryStore::default()));
    let janela = ScriptedPrompter::new(&[]);
    assert_eq!(unlock(&chaveiro, &janela, 5), Outcome::Unlocked);
    assert!(janela.asked().is_empty());
}
