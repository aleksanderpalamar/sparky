use crate::popup_grab::{Above, dismisses, trusted, visible};

#[test]
fn clique_dentro_do_menu_nao_fecha() {
    assert!(!dismisses(Some(&2), &[1, 2], Above::Nothing));
}

#[test]
fn clique_em_outra_superficie_fecha_o_menu() {
    assert!(dismisses(Some(&9), &[1, 2], Above::Nothing));
}

#[test]
fn clique_no_vazio_fecha_o_menu() {
    assert!(dismisses(None, &[1, 2], Above::Nothing));
}

#[test]
fn com_tela_exclusiva_aberta_o_clique_nao_fecha_o_menu() {
    assert!(!dismisses(Some(&9), &[1, 2], Above::ExclusiveLayer));
    assert!(!dismisses(None, &[1, 2], Above::ExclusiveLayer));
}

#[test]
fn grab_de_quem_tem_o_teclado_ou_o_ponteiro_e_aceito() {
    assert!(trusted(Some(&3), Some(&3), Some(&7)));
    assert!(trusted(Some(&3), Some(&7), Some(&3)));
}

#[test]
fn grab_de_cliente_em_segundo_plano_e_recusado() {
    assert!(!trusted(Some(&3), Some(&7), Some(&8)));
    assert!(!trusted(Some(&3), None, None));
    assert!(!trusted::<i32>(None, None, None));
}

#[test]
fn menu_de_janela_que_saiu_da_tela_perde_o_grab() {
    assert!(visible(&2, &[1, 2, 3]));
    assert!(!visible(&9, &[1, 2, 3]));
}
