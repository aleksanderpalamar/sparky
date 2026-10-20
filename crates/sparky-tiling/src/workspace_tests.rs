use crate::gaps::Gaps;
use crate::geometry::Rect;
use crate::ids::WindowId;
use crate::workspace::{Fullscreen, Layer, Placement, Screen, Workspace};

const TELA: Screen = Screen {
    full: Rect::new(0, 0, 1920, 1080),
    area: Rect::new(0, 26, 1920, 1054),
};

fn area_com(ids: &[u64]) -> Workspace {
    let mut workspace = Workspace::default();
    for id in ids {
        workspace.add_tiled(WindowId(*id), TELA.area);
    }
    workspace
}

fn posicao(workspace: &Workspace, id: u64) -> Placement {
    let found = workspace
        .arrange(TELA, Gaps::SPARKY)
        .into_iter()
        .find(|placement| placement.window == WindowId(id));
    let Some(placement) = found else {
        panic!("janela {id} sem posição");
    };
    placement
}

#[test]
fn janela_sozinha_respeita_a_barra_e_as_folgas() {
    let placement = posicao(&area_com(&[1]), 1);
    assert_eq!(placement.frame, Rect::new(10, 36, 1900, 1034));
    assert_eq!(placement.layer, Layer::Tiled);
    assert!(placement.focused);
}

#[test]
fn a_janela_nova_fica_com_o_foco() {
    let workspace = area_com(&[1, 2]);
    assert_eq!(workspace.focused(), Some(WindowId(2)));
    assert!(!posicao(&workspace, 1).focused);
}

#[test]
fn a_janela_nova_divide_a_que_esta_em_foco() {
    let mut workspace = area_com(&[1, 2]);
    workspace.focus(WindowId(1));
    workspace.add_tiled(WindowId(3), TELA.area);
    assert_eq!(posicao(&workspace, 3).frame.x, 10);
    assert_eq!(posicao(&workspace, 2).frame.height, 1034);
}

#[test]
fn fechar_a_janela_em_foco_devolve_o_foco_para_a_anterior() {
    let mut workspace = area_com(&[1, 2]);
    assert!(workspace.remove(WindowId(2)));
    assert_eq!(workspace.focused(), Some(WindowId(1)));
    assert_eq!(posicao(&workspace, 1).frame.width, 1900);
}

#[test]
fn flutuar_tira_do_mosaico_e_centraliza() {
    let mut workspace = area_com(&[1, 2]);
    workspace.toggle_floating(WindowId(2), TELA.area, (875, 600));
    let floating = posicao(&workspace, 2);
    assert_eq!(floating.layer, Layer::Floating);
    assert_eq!(floating.frame, Rect::new(522, 253, 875, 600));
    assert_eq!(posicao(&workspace, 1).frame.width, 1900);
}

#[test]
fn flutuar_de_novo_volta_para_o_mosaico() {
    let mut workspace = area_com(&[1, 2]);
    workspace.toggle_floating(WindowId(2), TELA.area, (875, 600));
    workspace.toggle_floating(WindowId(2), TELA.area, (875, 600));
    assert_eq!(posicao(&workspace, 2).layer, Layer::Tiled);
    assert_eq!(posicao(&workspace, 2).frame.width, 945);
}

#[test]
fn as_flutuantes_ficam_por_cima_das_lado_a_lado() {
    let mut workspace = area_com(&[1]);
    workspace.add_floating(WindowId(2), Rect::new(100, 100, 300, 200));
    workspace.add_tiled(WindowId(3), TELA.area);
    let order: Vec<WindowId> = workspace.arrange(TELA, Gaps::SPARKY).iter().map(|p| p.window).collect();
    assert_eq!(order.last(), Some(&WindowId(2)));
}

#[test]
fn tela_cheia_cobre_o_monitor_inteiro_por_cima_de_tudo() {
    let mut workspace = area_com(&[1, 2]);
    workspace.toggle_fullscreen(WindowId(1), Fullscreen::Monitor);
    let placements = workspace.arrange(TELA, Gaps::SPARKY);
    let Some(last) = placements.last() else {
        panic!("sem posições");
    };
    assert_eq!(last.window, WindowId(1));
    assert_eq!(last.frame, TELA.full);
    assert_eq!(last.layer, Layer::Fullscreen(Fullscreen::Monitor));
}

#[test]
fn largura_total_ocupa_a_area_com_a_folga_de_fora() {
    let mut workspace = area_com(&[1, 2]);
    workspace.toggle_fullscreen(WindowId(2), Fullscreen::Maximized);
    assert_eq!(posicao(&workspace, 2).frame, Rect::new(10, 36, 1900, 1034));
}

#[test]
fn alternar_a_tela_cheia_duas_vezes_desfaz() {
    let mut workspace = area_com(&[1]);
    workspace.toggle_fullscreen(WindowId(1), Fullscreen::Monitor);
    workspace.toggle_fullscreen(WindowId(1), Fullscreen::Monitor);
    assert_eq!(posicao(&workspace, 1).layer, Layer::Tiled);
}

#[test]
fn fechar_a_janela_em_tela_cheia_limpa_a_tela_cheia() {
    let mut workspace = area_com(&[1, 2]);
    workspace.toggle_fullscreen(WindowId(2), Fullscreen::Monitor);
    workspace.remove(WindowId(2));
    assert_eq!(posicao(&workspace, 1).layer, Layer::Tiled);
}

#[test]
fn redimensionar_flutuante_muda_a_caixa() {
    let mut workspace = Workspace::default();
    workspace.add_floating(WindowId(1), Rect::new(0, 0, 300, 200));
    workspace.resize(WindowId(1), (100, -50), TELA.area);
    assert_eq!(posicao(&workspace, 1).frame, Rect::new(0, 0, 400, 150));
}

#[test]
fn mover_flutuante_troca_a_caixa_inteira() {
    let mut workspace = Workspace::default();
    workspace.add_floating(WindowId(1), Rect::new(0, 0, 300, 200));
    workspace.set_floating_frame(WindowId(1), Rect::new(50, 60, 300, 200));
    assert_eq!(posicao(&workspace, 1).frame, Rect::new(50, 60, 300, 200));
}

#[test]
fn area_sem_janelas_esta_vazia() {
    let mut workspace = area_com(&[1]);
    workspace.remove(WindowId(1));
    assert!(workspace.is_empty());
    assert_eq!(workspace.focused(), None);
}
