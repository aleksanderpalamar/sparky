use crate::groups::{Groups, Leaving};
use crate::ids::WindowId;

fn grupo(ids: &[u64]) -> Groups {
    let mut groups = Groups::default();
    groups.create(WindowId(ids[0]));
    for pair in ids.windows(2) {
        groups.join(WindowId(pair[0]), WindowId(pair[1]));
    }
    groups.select(WindowId(ids[0]), 0);
    groups
}

fn membros(groups: &Groups, id: u64) -> Vec<u64> {
    groups.of(WindowId(id)).map(|group| group.members().iter().map(|w| w.0).collect()).unwrap_or_default()
}

#[test]
fn entrar_no_grupo_poe_depois_da_ativa_e_vira_a_ativa() {
    let mut groups = Groups::default();
    groups.create(WindowId(1));
    assert!(groups.join(WindowId(1), WindowId(2)));
    assert_eq!(membros(&groups, 1), [1, 2]);
    assert_eq!(groups.of(WindowId(1)).map(|group| group.active()), Some(WindowId(2)));
    assert!(groups.is_hidden(WindowId(1)));
}

#[test]
fn so_entra_em_grupo_que_existe() {
    let mut groups = Groups::default();
    assert!(!groups.join(WindowId(1), WindowId(2)));
}

#[test]
fn andar_no_grupo_da_a_volta_e_diz_quem_sai_e_quem_entra() {
    let mut groups = grupo(&[1, 2, 3]);
    assert_eq!(groups.step(WindowId(1), 1), Some((WindowId(1), WindowId(2))));
    assert_eq!(groups.step(WindowId(2), -2), Some((WindowId(2), WindowId(3))));
    assert_eq!(groups.select(WindowId(3), 0), Some((WindowId(3), WindowId(1))));
    assert_eq!(groups.select(WindowId(1), 9), None);
}

#[test]
fn sair_da_ativa_passa_a_folha_para_a_vizinha() {
    let mut groups = grupo(&[1, 2, 3]);
    assert_eq!(groups.leave(WindowId(1)), Leaving::Replaced(WindowId(2)));
    assert_eq!(membros(&groups, 2), [2, 3]);
}

#[test]
fn sair_escondida_nao_mexe_na_folha() {
    let mut groups = grupo(&[1, 2]);
    assert_eq!(groups.leave(WindowId(2)), Leaving::Hidden);
    assert_eq!(membros(&groups, 1), [1]);
}

#[test]
fn a_ultima_a_sair_desfaz_o_grupo() {
    let mut groups = grupo(&[1]);
    assert_eq!(groups.leave(WindowId(1)), Leaving::Last);
    assert!(groups.of(WindowId(1)).is_none());
    assert_eq!(groups.leave(WindowId(1)), Leaving::NotGrouped);
}

#[test]
fn desfazer_devolve_as_escondidas() {
    let mut groups = grupo(&[1, 2, 3]);
    assert_eq!(groups.dissolve(WindowId(2)), vec![WindowId(2), WindowId(3)]);
    assert!(groups.of(WindowId(1)).is_none());
}

#[test]
fn sair_uma_escondida_antes_da_ativa_mantem_a_mesma_ativa() {
    let mut groups = grupo(&[1, 2, 3]);
    groups.select(WindowId(1), 2);
    assert_eq!(groups.leave(WindowId(1)), Leaving::Hidden);
    assert_eq!(groups.of(WindowId(3)).map(|group| group.active()), Some(WindowId(3)));
}
