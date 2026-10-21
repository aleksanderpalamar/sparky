use crate::desktop::{Desktop, Style};
use crate::desktop_windows::Opening;
use crate::geometry::{Direction, Rect};
use crate::ids::{WindowId, WorkspaceId};
use crate::workspace::{Fullscreen, Layer, Placement};

fn desktop_com(ids: &[u64]) -> Desktop {
    let mut desktop = Desktop::new(Style::SPARKY);
    desktop.add_monitor("HDMI-A-1", Rect::new(0, 0, 1920, 1080));
    for id in ids {
        desktop.add_window(WindowId(*id), Opening::Tiled);
    }
    desktop
}

fn posicoes(desktop: &Desktop) -> Vec<Placement> {
    desktop.arrange().into_iter().flat_map(|(_, placements)| placements).collect()
}

fn posicao(desktop: &Desktop, id: u64) -> Option<Placement> {
    posicoes(desktop).into_iter().find(|placement| placement.window == WindowId(id))
}

#[test]
fn a_janela_aberta_entra_na_area_ativa_com_foco() {
    let desktop = desktop_com(&[1]);
    assert_eq!(desktop.workspace_of(WindowId(1)), Some(WorkspaceId(1)));
    assert_eq!(desktop.focused_window(), Some(WindowId(1)));
}

#[test]
fn janela_que_pede_para_flutuar_abre_centralizada() {
    let mut desktop = desktop_com(&[]);
    desktop.add_window(WindowId(1), Opening::Floating(400, 300));
    let Some(placement) = posicao(&desktop, 1) else {
        panic!("sem posição");
    };
    assert_eq!(placement.layer, Layer::Floating);
    assert_eq!(placement.frame, Rect::new(760, 390, 400, 300));
}

#[test]
fn o_foco_anda_pela_direcao() {
    let mut desktop = desktop_com(&[1, 2, 3]);
    assert_eq!(desktop.focus_direction(Direction::Left), Some(WindowId(1)));
    assert_eq!(desktop.focus_direction(Direction::Right), Some(WindowId(2)));
    assert_eq!(desktop.focus_direction(Direction::Down), Some(WindowId(3)));
}

#[test]
fn sem_vizinho_o_foco_passa_para_o_monitor_ao_lado() {
    let mut desktop = desktop_com(&[1]);
    desktop.add_monitor("DP-1", Rect::new(1920, 0, 1920, 1080));
    assert_eq!(desktop.focus_direction(Direction::Right), None);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(2)));
}

#[test]
fn trocar_pela_direcao_troca_as_caixas() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.swap_direction(Direction::Left);
    assert_eq!(posicao(&desktop, 2).map(|p| p.frame.x), Some(10));
}

#[test]
fn fechar_a_janela_tira_do_mosaico() {
    let mut desktop = desktop_com(&[1, 2]);
    assert!(desktop.remove_window(WindowId(2)));
    assert_eq!(desktop.focused_window(), Some(WindowId(1)));
    assert_eq!(posicoes(&desktop).len(), 1);
}

#[test]
fn mover_para_outra_area_sem_seguir_fica_onde_esta() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_workspace(WindowId(2), WorkspaceId(3), false);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(1)));
    assert_eq!(desktop.workspace_of(WindowId(2)), Some(WorkspaceId(3)));
    assert_eq!(posicoes(&desktop).len(), 1);
}

#[test]
fn mover_para_outra_area_seguindo_leva_o_foco_junto() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_workspace(WindowId(2), WorkspaceId(3), true);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(3)));
    assert_eq!(desktop.focused_window(), Some(WindowId(2)));
}

#[test]
fn a_flutuante_continua_flutuante_na_outra_area() {
    let mut desktop = desktop_com(&[1]);
    desktop.toggle_floating();
    desktop.move_to_workspace(WindowId(1), WorkspaceId(2), true);
    assert_eq!(posicao(&desktop, 1).map(|p| p.layer), Some(Layer::Floating));
}

#[test]
fn focar_janela_de_outra_area_troca_de_area() {
    let mut desktop = desktop_com(&[1]);
    desktop.focus_workspace(WorkspaceId(2));
    desktop.add_window(WindowId(2), Opening::Tiled);
    desktop.focus_window(WindowId(1));
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(1)));
}

#[test]
fn as_alternancias_valem_para_a_janela_em_foco() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_fullscreen(Fullscreen::Monitor);
    assert_eq!(posicao(&desktop, 2).map(|p| p.layer), Some(Layer::Fullscreen(Fullscreen::Monitor)));
    desktop.toggle_fullscreen(Fullscreen::Monitor);
    desktop.toggle_split();
    assert_eq!(posicao(&desktop, 2).map(|p| p.frame.y), Some(545));
}

#[test]
fn diminuir_a_janela_em_foco_move_a_divisao() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.resize_focused((-192, 0));
    assert_eq!(posicao(&desktop, 2).map(|p| p.frame.width), Some(753));
    assert_eq!(posicao(&desktop, 1).map(|p| p.frame.width), Some(1137));
}

#[test]
fn arrastar_a_flutuante_muda_a_caixa() {
    let mut desktop = desktop_com(&[]);
    desktop.add_window(WindowId(1), Opening::Floating(300, 200));
    desktop.set_floating_frame(WindowId(1), Rect::new(5, 5, 300, 200));
    assert_eq!(posicao(&desktop, 1).map(|p| p.frame), Some(Rect::new(5, 5, 300, 200)));
}

#[test]
fn trocar_duas_janelas_do_mosaico_troca_as_caixas() {
    let mut desktop = desktop_com(&[1, 2]);
    let (Some(antes_1), Some(antes_2)) = (posicao(&desktop, 1), posicao(&desktop, 2)) else {
        panic!("sem posição");
    };
    assert!(desktop.swap_windows(WindowId(1), WindowId(2)));
    assert_eq!(posicao(&desktop, 1).map(|placement| placement.frame), Some(antes_2.frame));
    assert_eq!(posicao(&desktop, 2).map(|placement| placement.frame), Some(antes_1.frame));
}

#[test]
fn nao_troca_com_flutuante_nem_entre_areas() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.add_window(WindowId(3), Opening::Floating(400, 300));
    assert!(!desktop.swap_windows(WindowId(1), WindowId(3)));
    desktop.move_to_workspace(WindowId(2), WorkspaceId(2), false);
    assert!(!desktop.swap_windows(WindowId(1), WindowId(2)));
    assert!(!desktop.swap_windows(WindowId(1), WindowId(1)));
}
