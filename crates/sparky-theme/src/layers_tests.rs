use crate::layers::{WINDOW, card};
use crate::palette::Mode;

#[test]
fn a_janela_com_vidro_deixa_passar_um_quarto_do_fundo() {
    assert_eq!(WINDOW, 0.76);
}

#[test]
fn cartoes_dentro_da_janela_ficam_opacos_no_nivel_padrao() {
    assert_eq!(card(Mode::Dark, WINDOW), 1.0);
    assert_eq!(card(Mode::Light, WINDOW), 1.0);
}

#[test]
fn cartoes_somam_mais_opacidade_no_tema_escuro_que_no_claro() {
    assert!((card(Mode::Dark, 0.6) - 0.9).abs() < 1e-6);
    assert!((card(Mode::Light, 0.6) - 0.85).abs() < 1e-6);
}

#[test]
fn quanto_mais_transparente_a_janela_mais_o_texto_se_afasta_do_fundo() {
    assert!((crate::layers::spread(WINDOW) - 0.24).abs() < 1e-6);
    assert_eq!(crate::layers::spread(1.0), 0.0);
}
