use sparky_tiling::Direction;

use crate::action::{Action, Bind};
use crate::combo::Combo;
use crate::defaults::sparky;
use crate::schema::Apps;

fn padrao() -> Vec<Bind> {
    sparky(&Apps::default(), "kitty")
}

fn acao(keys: &str) -> Option<Action> {
    padrao().into_iter().find(|bind| bind.keys == keys).map(|bind| bind.action)
}

#[test]
fn todas_as_combinacoes_padrao_sao_validas() {
    for bind in padrao() {
        assert!(Combo::parse(&bind.keys).is_ok(), "combinação inválida: {}", bind.keys);
    }
}

#[test]
fn nenhuma_combinacao_padrao_se_repete() {
    let mut keys: Vec<String> = padrao().into_iter().map(|bind| bind.keys).collect();
    let total = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), total);
}

#[test]
fn super_enter_abre_o_terminal() {
    assert_eq!(acao("SUPER + RETURN"), Some(Action::Spawn { command: "kitty".to_owned() }));
}

#[test]
fn a_tecla_do_um_vai_para_a_area_um_e_a_do_zero_para_a_dez() {
    assert_eq!(acao("SUPER + code:10"), Some(Action::Workspace { workspace: 1 }));
    assert_eq!(acao("SUPER + code:19"), Some(Action::Workspace { workspace: 10 }));
}

#[test]
fn mover_sem_seguir_usa_alt() {
    assert_eq!(
        acao("SUPER + SHIFT + ALT + code:12"),
        Some(Action::MoveToWorkspace { workspace: 3, follow: false })
    );
}

#[test]
fn super_s_e_super_crase_alternam_o_rascunho() {
    assert_eq!(acao("SUPER + S"), Some(Action::ToggleScratchpad));
    assert_eq!(acao("SUPER + code:49"), Some(Action::ToggleScratchpad));
}

#[test]
fn super_alt_s_e_super_shift_crase_mandam_para_o_rascunho() {
    assert_eq!(acao("SUPER + ALT + S"), Some(Action::MoveToScratchpad));
    assert_eq!(acao("SUPER + SHIFT + code:49"), Some(Action::MoveToScratchpad));
}

#[test]
fn super_g_agrupa_e_super_alt_g_tira_do_grupo() {
    assert_eq!(acao("SUPER + G"), Some(Action::ToggleGroup));
    assert_eq!(acao("SUPER + ALT + G"), Some(Action::LeaveGroup));
}

#[test]
fn super_alt_com_seta_entra_no_grupo_vizinho() {
    assert_eq!(acao("SUPER + ALT + LEFT"), Some(Action::MoveIntoGroup { direction: Direction::Left }));
    assert_eq!(acao("SUPER + ALT + DOWN"), Some(Action::MoveIntoGroup { direction: Direction::Down }));
}

#[test]
fn as_abas_andam_por_tab_por_ctrl_com_seta_e_pelo_numero() {
    assert_eq!(acao("SUPER + ALT + TAB"), Some(Action::CycleGroup { step: 1 }));
    assert_eq!(acao("SUPER + ALT + SHIFT + TAB"), Some(Action::CycleGroup { step: -1 }));
    assert_eq!(acao("SUPER + CTRL + RIGHT"), Some(Action::CycleGroup { step: 1 }));
    assert_eq!(acao("SUPER + CTRL + LEFT"), Some(Action::CycleGroup { step: -1 }));
    assert_eq!(acao("SUPER + ALT + code:10"), Some(Action::SelectInGroup { index: 1 }));
    assert_eq!(acao("SUPER + ALT + code:14"), Some(Action::SelectInGroup { index: 5 }));
}

#[test]
fn print_captura_uma_regiao() {
    assert_eq!(acao("PRINT"), Some(Action::Spawn { command: "sparky-shot region".to_owned() }));
}

#[test]
fn super_ctrl_l_bloqueia_a_sessao_com_o_locker() {
    assert_eq!(acao("SUPER + CTRL + L"), Some(Action::Spawn { command: "swaylock".to_owned() }));
}
