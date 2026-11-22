use super::layout::{Layout, MENU_SPACE, PEEK};
use super::region::{Rect, Stage, region};

fn layout() -> Layout {
    Layout::new(64)
}

fn at(stage: Stage) -> Rect {
    let frame = layout().frame(4, false);
    region(&layout(), 4, frame, stage)
}

#[test]
fn oculto_so_a_faixa_da_borda_recebe_o_mouse() {
    let frame = layout().frame(4, false);
    let faixa = at(Stage::Tucked);
    assert_eq!((faixa.width, faixa.height), (408, PEEK as i32));
    assert_eq!(faixa.y + faixa.height, frame.size.1 as i32);
    assert!((faixa.x * 2 + faixa.width - frame.size.0 as i32).abs() <= 1);
}

#[test]
fn parado_so_o_dock_e_a_folga_ate_a_borda_recebem_o_mouse() {
    let parado = at(Stage::Resting);
    assert_eq!((parado.width, parado.height), (408, 117));
    let faixa = at(Stage::Tucked);
    assert!(parado.y <= faixa.y && parado.x <= faixa.x);
}

#[test]
fn ampliado_e_com_menu_a_area_cresce_para_cima_cobrindo_a_anterior() {
    let parado = at(Stage::Resting);
    let ampliado = at(Stage::Lifted);
    let menu = at(Stage::Menu);
    assert!(ampliado.y < parado.y && ampliado.width > parado.width);
    assert_eq!(menu.height, 117 + MENU_SPACE as i32);
    assert_eq!(ampliado.y + ampliado.height, parado.y + parado.height);
}
