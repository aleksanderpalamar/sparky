use crate::drag::{Edges, Horizontal, MINIMUM, Vertical, moved, resized};
use crate::geometry::Rect;

const JANELA: Rect = Rect::new(100, 100, 400, 300);

fn canto(horizontal: Horizontal, vertical: Vertical) -> Edges {
    Edges { horizontal, vertical }
}

#[test]
fn o_canto_mais_perto_do_ponteiro_e_o_que_estica() {
    assert_eq!(Edges::nearest(JANELA, 110, 110), canto(Horizontal::Left, Vertical::Top));
    assert_eq!(Edges::nearest(JANELA, 480, 390), canto(Horizontal::Right, Vertical::Bottom));
    assert_eq!(Edges::nearest(JANELA, 480, 110), canto(Horizontal::Right, Vertical::Top));
}

#[test]
fn mover_desloca_sem_mudar_o_tamanho() {
    assert_eq!(moved(JANELA, (-30, 50)), Rect::new(70, 150, 400, 300));
}

#[test]
fn puxar_o_canto_de_baixo_a_direita_cresce_para_fora() {
    let novo = resized(JANELA, canto(Horizontal::Right, Vertical::Bottom), (50, 20));
    assert_eq!(novo, Rect::new(100, 100, 450, 320));
}

#[test]
fn puxar_o_canto_de_cima_a_esquerda_mantem_o_canto_oposto_parado() {
    let novo = resized(JANELA, canto(Horizontal::Left, Vertical::Top), (-50, -20));
    assert_eq!(novo, Rect::new(50, 80, 450, 320));
    assert_eq!((novo.right(), novo.bottom()), (JANELA.right(), JANELA.bottom()));
}

#[test]
fn a_janela_nao_encolhe_abaixo_do_minimo() {
    let novo = resized(JANELA, canto(Horizontal::Left, Vertical::Top), (1000, 1000));
    assert_eq!((novo.width, novo.height), MINIMUM);
    assert_eq!((novo.right(), novo.bottom()), (JANELA.right(), JANELA.bottom()));
}

#[test]
fn borda_de_um_lado_so_estica_numa_direcao() {
    let lateral = canto(Horizontal::Right, Vertical::Neither);
    assert_eq!(resized(JANELA, lateral, (40, 999)), Rect::new(100, 100, 440, 300));
    assert_eq!(lateral.growth((40, 999)), (40, 0));
}

#[test]
fn no_mosaico_puxar_para_a_esquerda_pelo_lado_esquerdo_cresce() {
    assert_eq!(canto(Horizontal::Left, Vertical::Top).growth((-30, 10)), (30, -10));
}
