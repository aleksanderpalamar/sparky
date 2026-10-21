use std::time::Duration;

use sparky_ipc::SessionAction;

use crate::session::departure::{CLOSE_GRACE, Closing, Farewell, LOCK_GRACE, Plan, Sleep, allowed, closing, plan, sleep, stuck_message};

#[test]
fn cada_acao_do_menu_vira_um_plano() {
    assert_eq!(plan(SessionAction::Lock), Plan::Lock);
    assert_eq!(plan(SessionAction::Suspend), Plan::Suspend);
    assert_eq!(plan(SessionAction::Logout), Plan::Leave(Farewell::Logout));
    assert_eq!(plan(SessionAction::Reboot), Plan::Leave(Farewell::Reboot));
    assert_eq!(plan(SessionAction::Poweroff), Plan::Leave(Farewell::Poweroff));
}

#[test]
fn reiniciar_e_desligar_vao_pelo_systemctl_e_sair_so_encerra_o_sparky() {
    assert_eq!(Farewell::Logout.power(), None);
    assert_eq!(Farewell::Reboot.power(), Some("reboot"));
    assert_eq!(Farewell::Poweroff.power(), Some("poweroff"));
}

#[test]
fn aninhado_so_bloqueia_e_sai_para_nunca_desligar_a_maquina_de_verdade() {
    assert!(allowed(Plan::Lock, false));
    assert!(allowed(Plan::Leave(Farewell::Logout), false));
    assert!(!allowed(Plan::Suspend, false));
    assert!(!allowed(Plan::Leave(Farewell::Reboot), false));
    assert!(!allowed(Plan::Leave(Farewell::Poweroff), false));
    assert!(allowed(Plan::Leave(Farewell::Poweroff), true));
}

#[test]
fn sai_quando_as_janelas_fecham_e_desiste_se_alguma_nao_fecha() {
    assert_eq!(closing(3, Duration::ZERO), Closing::Waiting);
    assert_eq!(closing(0, Duration::from_millis(300)), Closing::Done);
    assert_eq!(closing(0, CLOSE_GRACE * 2), Closing::Done);
    assert_eq!(closing(2, CLOSE_GRACE), Closing::Stuck(2));
}

#[test]
fn suspende_depois_de_trancar_ou_quando_o_bloqueio_demora_demais() {
    assert_eq!(sleep(false, Duration::ZERO), Sleep::Waiting);
    assert_eq!(sleep(true, Duration::ZERO), Sleep::Now);
    assert_eq!(sleep(false, LOCK_GRACE), Sleep::Now);
}

#[test]
fn o_aviso_explica_como_continuar() {
    assert_eq!(
        stuck_message(1),
        "Ainda há 1 janela aberta. Salve e feche, ou saia à força com SUPER + SHIFT + ESC."
    );
    assert_eq!(
        stuck_message(3),
        "Ainda há 3 janelas abertas. Salve e feche, ou saia à força com SUPER + SHIFT + ESC."
    );
}
