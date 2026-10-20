use crate::desktop::{Desktop, Style};
use crate::desktop_windows::Opening;
use crate::geometry::{Direction, Rect};
use crate::ids::{WindowId, WorkspaceId};
use crate::workspace::Screen;

const ESQUERDA: Rect = Rect::new(0, 0, 1920, 1080);
const DIREITA: Rect = Rect::new(1920, 0, 2560, 1440);

fn um_monitor() -> Desktop {
    let mut desktop = Desktop::new(Style::SPARKY);
    desktop.add_monitor("HDMI-A-1", ESQUERDA);
    desktop
}

fn dois_monitores() -> Desktop {
    let mut desktop = um_monitor();
    desktop.add_monitor("DP-1", DIREITA);
    desktop
}

fn ativas(desktop: &Desktop) -> Vec<(String, u32)> {
    desktop
        .monitors()
        .iter()
        .map(|monitor| (monitor.name.clone(), monitor.active.0))
        .collect()
}

#[test]
fn cada_monitor_novo_ganha_a_primeira_area_livre() {
    let desktop = dois_monitores();
    assert_eq!(ativas(&desktop), vec![("HDMI-A-1".to_owned(), 1), ("DP-1".to_owned(), 2)]);
}

#[test]
fn ir_para_uma_area_nova_cria_no_monitor_em_foco() {
    let mut desktop = dois_monitores();
    desktop.focus_workspace(WorkspaceId(5));
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(5)));
    assert_eq!(ativas(&desktop)[0], ("HDMI-A-1".to_owned(), 5));
}

#[test]
fn ir_para_a_area_visivel_em_outro_monitor_troca_o_foco_de_monitor() {
    let mut desktop = dois_monitores();
    desktop.focus_workspace(WorkspaceId(2));
    assert_eq!(desktop.focused_monitor().map(|m| m.name.as_str()), Some("DP-1"));
    assert_eq!(ativas(&desktop)[0].1, 1);
}

#[test]
fn area_vazia_que_sai_da_tela_deixa_de_existir() {
    let mut desktop = um_monitor();
    desktop.focus_workspace(WorkspaceId(3));
    desktop.focus_workspace(WorkspaceId(4));
    let ids: Vec<u32> = desktop.summaries().iter().map(|summary| summary.id.0).collect();
    assert_eq!(ids, vec![4]);
}

#[test]
fn area_com_janela_continua_existindo_fora_da_tela() {
    let mut desktop = um_monitor();
    desktop.add_window(WindowId(1), Opening::Tiled);
    desktop.focus_workspace(WorkspaceId(2));
    let summaries = desktop.summaries();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].windows, 1);
    assert!(!summaries[0].visible);
    assert!(summaries[1].focused);
}

#[test]
fn a_proxima_area_aberta_da_a_volta() {
    let mut desktop = um_monitor();
    desktop.add_window(WindowId(1), Opening::Tiled);
    desktop.focus_workspace(WorkspaceId(3));
    desktop.add_window(WindowId(2), Opening::Tiled);
    desktop.focus_relative(1);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(1)));
    desktop.focus_relative(-1);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(3)));
}

#[test]
fn volta_para_a_ultima_area_usada() {
    let mut desktop = um_monitor();
    desktop.focus_workspace(WorkspaceId(4));
    desktop.focus_previous();
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(1)));
}

#[test]
fn trocar_de_monitor_da_a_volta() {
    let mut desktop = dois_monitores();
    desktop.cycle_monitor(1);
    assert_eq!(desktop.focused_monitor().map(|m| m.name.as_str()), Some("DP-1"));
    desktop.cycle_monitor(1);
    assert_eq!(desktop.focused_monitor().map(|m| m.name.as_str()), Some("HDMI-A-1"));
}

#[test]
fn o_monitor_sob_o_ponteiro_ganha_o_foco() {
    let mut desktop = dois_monitores();
    desktop.focus_monitor_at(2000, 100);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(2)));
}

#[test]
fn o_monitor_a_direita_e_achado_pela_direcao() {
    let desktop = dois_monitores();
    assert_eq!(desktop.monitor_towards(Direction::Right).as_deref(), Some("DP-1"));
    assert_eq!(desktop.monitor_towards(Direction::Left), None);
}

#[test]
fn as_areas_do_monitor_removido_passam_para_outro() {
    let mut desktop = dois_monitores();
    desktop.focus_workspace(WorkspaceId(2));
    desktop.add_window(WindowId(7), Opening::Tiled);
    desktop.remove_monitor("DP-1");
    desktop.focus_workspace(WorkspaceId(2));
    assert_eq!(desktop.focused_window(), Some(WindowId(7)));
    assert_eq!(ativas(&desktop), vec![("HDMI-A-1".to_owned(), 2)]);
}

#[test]
fn a_barra_reserva_o_topo_da_area() {
    let mut desktop = um_monitor();
    let area = Rect::new(0, 26, 1920, 1054);
    desktop.set_screen("HDMI-A-1", Screen { full: ESQUERDA, area });
    desktop.add_window(WindowId(1), Opening::Tiled);
    let arranged = desktop.arrange();
    assert_eq!(arranged[0].1[0].frame, Rect::new(10, 36, 1900, 1034));
}

#[test]
fn sem_monitor_nada_acontece() {
    let mut desktop = Desktop::new(Style::SPARKY);
    desktop.add_window(WindowId(1), Opening::Tiled);
    desktop.focus_workspace(WorkspaceId(2));
    assert_eq!(desktop.focused_window(), None);
    assert!(desktop.arrange().is_empty());
}
