use crate::domain::guard::{Guard, Input, Order, Screen};
use crate::domain::inhibitions::Hold;

fn guarda() -> Guard {
    Guard::start().0
}

#[test]
fn ao_comecar_segura_a_suspensao_e_avisa_que_a_tela_esta_aberta() {
    assert_eq!(
        Guard::start().1,
        vec![Order::Hold, Order::Hint(Screen::Open)]
    );
}

#[test]
fn os_pedidos_do_logind_viram_pedidos_ao_compositor() {
    let mut guarda = guarda();
    assert_eq!(guarda.handle(Input::LockAsked), vec![Order::Lock]);
    assert_eq!(guarda.handle(Input::UnlockAsked), vec![Order::Unlock]);
}

#[test]
fn a_dica_do_logind_acompanha_o_bloqueio() {
    let mut guarda = guarda();
    assert_eq!(
        guarda.handle(Input::Locked),
        vec![Order::Hint(Screen::Locked)]
    );
    assert_eq!(
        guarda.handle(Input::Unlocked),
        vec![Order::Hint(Screen::Open)]
    );
}

#[test]
fn antes_de_suspender_bloqueia_e_so_libera_depois_de_bloqueado() {
    let mut guarda = guarda();
    assert_eq!(
        guarda.handle(Input::Sleeping),
        vec![Order::Lock, Order::Watch]
    );
    assert_eq!(
        guarda.handle(Input::Locked),
        vec![Order::Hint(Screen::Locked), Order::Release]
    );
}

#[test]
fn com_a_tela_ja_bloqueada_libera_a_suspensao_na_hora() {
    let mut guarda = guarda();
    guarda.handle(Input::Locked);
    assert_eq!(guarda.handle(Input::Sleeping), vec![Order::Release]);
}

#[test]
fn se_o_bloqueio_demora_demais_a_suspensao_segue_assim_mesmo() {
    let mut guarda = guarda();
    guarda.handle(Input::Sleeping);
    assert_eq!(guarda.handle(Input::Expired), vec![Order::Release]);
    assert_eq!(
        guarda.handle(Input::Locked),
        vec![Order::Hint(Screen::Locked)]
    );
}

#[test]
fn o_prazo_fora_da_suspensao_nao_faz_nada() {
    let mut guarda = guarda();
    assert_eq!(guarda.handle(Input::Expired), vec![]);
}

#[test]
fn aviso_repetido_de_suspensao_nao_bloqueia_de_novo() {
    let mut guarda = guarda();
    guarda.handle(Input::Sleeping);
    assert_eq!(guarda.handle(Input::Sleeping), vec![]);
}

#[test]
fn ao_acordar_segura_a_proxima_suspensao() {
    let mut guarda = guarda();
    guarda.handle(Input::Sleeping);
    guarda.handle(Input::Locked);
    assert_eq!(guarda.handle(Input::Woke), vec![Order::Hold]);
    assert_eq!(guarda.handle(Input::Sleeping), vec![Order::Release]);
}

#[test]
fn acordar_sem_ter_dormido_nao_pede_outro_inibidor() {
    let mut guarda = guarda();
    assert_eq!(guarda.handle(Input::Woke), vec![]);
}

#[test]
fn os_pedidos_de_tela_acesa_vao_ao_compositor() {
    let mut guarda = guarda();
    assert_eq!(
        guarda.handle(Input::Idle(Hold::Held)),
        vec![Order::Idle(Hold::Held)]
    );
    assert_eq!(
        guarda.handle(Input::Idle(Hold::Free)),
        vec![Order::Idle(Hold::Free)]
    );
}
