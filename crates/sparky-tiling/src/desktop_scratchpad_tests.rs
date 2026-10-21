use crate::desktop::{Desktop, Style};
use crate::desktop_windows::Opening;
use crate::geometry::Rect;
use crate::ids::{WindowId, WorkspaceId};
use crate::workspace::Placement;

fn desktop_com(ids: &[u64]) -> Desktop {
    let mut desktop = Desktop::new(Style::SPARKY);
    desktop.add_monitor("HDMI-A-1", Rect::new(0, 0, 1920, 1080));
    for id in ids {
        desktop.add_window(WindowId(*id), Opening::Tiled);
    }
    desktop
}

fn na_tela(desktop: &Desktop) -> Vec<u64> {
    desktop
        .arrange()
        .into_iter()
        .flat_map(|(_, placements)| placements)
        .map(|placement: Placement| placement.window.0)
        .collect()
}

#[test]
fn mandar_para_o_rascunho_tira_a_janela_da_tela() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    assert_eq!(desktop.workspace_of(WindowId(2)), Some(WorkspaceId::SCRATCHPAD));
    assert_eq!(na_tela(&desktop), [1]);
    assert_eq!(desktop.focused_window(), Some(WindowId(1)));
}

#[test]
fn alternar_mostra_o_rascunho_por_cima_com_foco_e_esconde_de_novo() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    desktop.toggle_scratchpad();
    assert_eq!(na_tela(&desktop), [1, 2]);
    assert_eq!(desktop.focused_window(), Some(WindowId(2)));
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId::SCRATCHPAD));
    desktop.toggle_scratchpad();
    assert_eq!(na_tela(&desktop), [1]);
    assert_eq!(desktop.focused_window(), Some(WindowId(1)));
}

#[test]
fn janela_nova_com_o_rascunho_aberto_entra_nele() {
    let mut desktop = desktop_com(&[1]);
    desktop.toggle_scratchpad();
    desktop.add_window(WindowId(3), Opening::Tiled);
    assert_eq!(desktop.workspace_of(WindowId(3)), Some(WorkspaceId::SCRATCHPAD));
}

#[test]
fn trocar_de_area_esconde_o_rascunho() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    desktop.toggle_scratchpad();
    desktop.focus_workspace(WorkspaceId(2));
    assert_eq!(desktop.scratchpad_shown(), None);
    assert_eq!(desktop.active_workspace(), Some(WorkspaceId(2)));
}

#[test]
fn levar_a_janela_do_rascunho_para_uma_area_fecha_o_rascunho() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    desktop.toggle_scratchpad();
    desktop.move_to_workspace(WindowId(2), WorkspaceId(1), true);
    assert_eq!(desktop.workspace_of(WindowId(2)), Some(WorkspaceId(1)));
    assert_eq!(desktop.scratchpad_shown(), None);
    assert_eq!(desktop.focused_window(), Some(WindowId(2)));
}

#[test]
fn com_o_rascunho_aberto_a_area_de_baixo_fica_coberta() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    desktop.toggle_scratchpad();
    assert!(desktop.is_covered(WindowId(1)));
    assert!(!desktop.is_covered(WindowId(2)));
    desktop.toggle_scratchpad();
    assert!(!desktop.is_covered(WindowId(1)));
}

#[test]
fn a_barra_nao_ve_o_rascunho_e_continua_marcando_a_area_de_baixo() {
    let mut desktop = desktop_com(&[1, 2]);
    desktop.move_to_scratchpad(WindowId(2));
    desktop.toggle_scratchpad();
    let resumo = desktop.summaries();
    assert!(resumo.iter().all(|area| area.id.is_numbered()));
    assert!(resumo.iter().any(|area| area.id == WorkspaceId(1) && area.focused));
}

#[test]
fn o_rascunho_abre_no_monitor_em_foco_levando_as_flutuantes() {
    let mut desktop = desktop_com(&[]);
    desktop.add_monitor("DP-1", Rect::new(1920, 0, 1920, 1080));
    desktop.add_window(WindowId(5), Opening::Floating(400, 300));
    desktop.move_to_scratchpad(WindowId(5));
    desktop.toggle_scratchpad();
    desktop.focus_monitor("DP-1");
    desktop.toggle_scratchpad();
    assert_eq!(desktop.scratchpad_shown(), Some("DP-1"));
    let quadro = desktop.arrange().into_iter().find(|(nome, _)| nome == "DP-1").and_then(|(_, p)| p.first().copied());
    assert_eq!(quadro.map(|placement| placement.frame), Some(Rect::new(2442, 240, 875, 600)));
}

#[test]
fn tirar_o_monitor_do_rascunho_esconde_o_rascunho() {
    let mut desktop = desktop_com(&[1]);
    desktop.add_monitor("DP-1", Rect::new(1920, 0, 1920, 1080));
    desktop.focus_monitor("DP-1");
    desktop.toggle_scratchpad();
    desktop.remove_monitor("DP-1");
    assert_eq!(desktop.scratchpad_shown(), None);
}
