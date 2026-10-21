use crate::desktop::{Desktop, Style};
use crate::desktop_windows::Opening;
use crate::geometry::{Direction, Rect};
use crate::groups::BAR_HEIGHT;
use crate::ids::WindowId;
use crate::workspace_groups::Tabs;

fn desktop_com(ids: &[u64]) -> Desktop {
    let mut desktop = Desktop::new(Style::SPARKY);
    desktop.add_monitor("HDMI-A-1", Rect::new(0, 0, 1920, 1080));
    for id in ids {
        desktop.add_window(WindowId(*id), Opening::Tiled);
    }
    desktop
}

fn na_tela(desktop: &Desktop) -> Vec<u64> {
    let mut ids: Vec<u64> = desktop.arrange().into_iter().flat_map(|(_, p)| p).map(|p| p.window.0).collect();
    ids.sort();
    ids
}

fn quadro(desktop: &Desktop, id: u64) -> Option<Rect> {
    desktop.arrange().into_iter().flat_map(|(_, p)| p).find(|p| p.window == WindowId(id)).map(|p| p.frame)
}

fn abas(desktop: &Desktop) -> Vec<Tabs> {
    desktop.tabs().into_iter().flat_map(|(_, tabs)| tabs).collect()
}

fn membros(tabs: &Tabs) -> Vec<u64> {
    tabs.members.iter().map(|window| window.0).collect()
}

#[test]
fn agrupar_abre_a_barra_de_abas_no_topo_do_quadro() {
    let mut desktop = desktop_com(&[1, 2]);
    let Some(antes) = quadro(&desktop, 2) else { panic!("sem quadro") };
    desktop.toggle_group();
    let barra = abas(&desktop);
    assert_eq!(barra.len(), 1);
    assert_eq!(membros(&barra[0]), [2]);
    assert_eq!(barra[0].frame, Rect::new(antes.x, antes.y, antes.width, BAR_HEIGHT));
    assert_eq!(quadro(&desktop, 2), Some(Rect::new(antes.x, antes.y + BAR_HEIGHT, antes.width, antes.height - BAR_HEIGHT)));
}

#[test]
fn janela_nova_entra_no_grupo_em_foco_e_ocupa_o_quadro() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    let antes = quadro(&desktop, 2);
    desktop.add_window(WindowId(3), Opening::Tiled);
    assert_eq!(na_tela(&desktop), [1, 3]);
    assert_eq!(quadro(&desktop, 3), antes);
    assert_eq!(desktop.focused_window(), Some(WindowId(3)));
    assert_eq!(membros(&abas(&desktop)[0]), [2, 3]);
    assert_eq!(abas(&desktop)[0].active, 1);
}

#[test]
fn andar_e_escolher_trocam_a_janela_visivel_do_grupo() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.add_window(WindowId(3), Opening::Tiled);
    desktop.cycle_group(1);
    assert_eq!(na_tela(&desktop), [1, 2]);
    assert_eq!(desktop.focused_window(), Some(WindowId(2)));
    desktop.select_in_group(1);
    assert_eq!(na_tela(&desktop), [1, 3]);
}

#[test]
fn tirar_do_grupo_devolve_um_quadro_proprio() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.add_window(WindowId(3), Opening::Tiled);
    desktop.leave_group();
    assert_eq!(na_tela(&desktop), [1, 2, 3]);
    assert_eq!(membros(&abas(&desktop)[0]), [2]);
    assert_eq!(desktop.focused_window(), Some(WindowId(3)));
}

#[test]
fn entrar_no_grupo_do_vizinho() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.focus_direction(Direction::Left);
    desktop.move_into_group(Direction::Right);
    assert_eq!(na_tela(&desktop), [1]);
    assert_eq!(membros(&abas(&desktop)[0]), [2, 1]);
    assert_eq!(quadro(&desktop, 1).map(|frame| frame.width), Some(1920 - 20));
}

#[test]
fn entrar_onde_nao_ha_grupo_nao_faz_nada() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_into_group(Direction::Left);
    assert_eq!(na_tela(&desktop), [1, 2]);
    assert!(abas(&desktop).is_empty());
}

#[test]
fn desfazer_o_grupo_espalha_as_janelas() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.add_window(WindowId(3), Opening::Tiled);
    desktop.toggle_group();
    assert_eq!(na_tela(&desktop), [1, 2, 3]);
    assert!(abas(&desktop).is_empty());
}

#[test]
fn fechar_a_ativa_mostra_a_vizinha_do_grupo() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.add_window(WindowId(3), Opening::Tiled);
    desktop.remove_window(WindowId(3));
    assert_eq!(na_tela(&desktop), [1, 2]);
    assert_eq!(membros(&abas(&desktop)[0]), [2]);
}

#[test]
fn flutuar_uma_janela_do_grupo_tira_ela_do_grupo() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.toggle_group();
    desktop.add_window(WindowId(3), Opening::Tiled);
    desktop.toggle_floating();
    assert_eq!(na_tela(&desktop), [1, 2, 3]);
    assert_eq!(membros(&abas(&desktop)[0]), [2]);
}
