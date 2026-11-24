use crate::popup_grab::dismisses;

#[test]
fn clique_dentro_do_menu_nao_fecha() {
    assert!(!dismisses(Some(&2), &[1, 2]));
}

#[test]
fn clique_em_outra_superficie_fecha_o_menu() {
    assert!(dismisses(Some(&9), &[1, 2]));
}

#[test]
fn clique_no_vazio_fecha_o_menu() {
    assert!(dismisses(None, &[1, 2]));
}
