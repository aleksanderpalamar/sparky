use super::fit::{EMERGENCY, FLOOR, capacity, fitted};
use super::layout::Layout;

#[test]
fn sem_largura_conhecida_vale_o_tamanho_escolhido() {
    assert_eq!(fitted(64, 40, None), Layout::new(64));
}

#[test]
fn quando_cabe_o_tamanho_escolhido_nao_muda() {
    assert_eq!(fitted(64, 4, Some(1920)), Layout::new(64));
}

#[test]
fn quando_nao_cabe_os_icones_diminuem_ate_caber_na_tela() {
    let layout = fitted(64, 20, Some(1366));
    assert!(layout.slot < 64.0 && layout.slot >= FLOOR as f32);
    assert!(layout.frame(20, false).size.0 <= 1366);
    assert!(Layout::new(layout.slot as i32 + 1).frame(20, false).size.0 > 1366);
}

#[test]
fn a_capacidade_e_quantos_apps_cabem_no_menor_tamanho() {
    let cabem = capacity(Some(1366));
    assert!(Layout::new(FLOOR).frame(cabem, false).size.0 <= 1366);
    assert!(Layout::new(FLOOR).frame(cabem + 1, false).size.0 > 1366);
    assert_eq!(capacity(None), usize::MAX);
}

#[test]
fn apps_demais_para_a_tela_ainda_cabem_no_tamanho_de_emergencia() {
    let layout = fitted(64, 30, Some(1100));
    assert!(layout.slot < FLOOR as f32 && layout.slot >= EMERGENCY as f32);
    assert!(layout.frame(30, false).size.0 <= 1100);
}
