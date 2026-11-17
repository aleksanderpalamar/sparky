use crate::domain::groups::{Group, Groups};

fn grupo(name: &str, apps: &[&str]) -> Group {
    Group {
        name: name.to_owned(),
        apps: apps.iter().map(|app| (*app).to_owned()).collect(),
    }
}

#[test]
fn criar_grupo_limpa_o_nome_e_recusa_vazio_ou_repetido() {
    let mut grupos = Groups::default();
    assert_eq!(grupos.create("  Meus   jogos\t"), Some(0));
    assert_eq!(grupos.list()[0].name, "Meus jogos");
    assert_eq!(grupos.create("   "), None);
    assert_eq!(grupos.create("meus JOGOS"), None);
    assert_eq!(grupos.create("Trabalho"), Some(1));
}

#[test]
fn renomear_aceita_o_proprio_nome_e_recusa_o_de_outro_grupo() {
    let mut grupos = Groups::new(vec![grupo("A", &[]), grupo("B", &[])]);
    assert!(grupos.rename(0, "a"));
    assert!(!grupos.rename(0, "B"));
    assert!(!grupos.rename(0, ""));
    assert!(!grupos.rename(5, "C"));
    assert_eq!(grupos.list()[0].name, "a");
}

#[test]
fn adicionar_nao_repete_e_tirar_avisa_se_mudou() {
    let mut grupos = Groups::new(vec![grupo("A", &[])]);
    assert!(grupos.add(0, "kitty.desktop"));
    assert!(!grupos.add(0, "kitty.desktop"));
    assert!(grupos.holds(0, "kitty.desktop"));
    assert!(grupos.take(0, "kitty.desktop"));
    assert!(!grupos.take(0, "kitty.desktop"));
    assert!(!grupos.add(3, "kitty.desktop"));
}

#[test]
fn sabe_se_o_aplicativo_esta_em_algum_grupo() {
    let mut grupos = Groups::new(vec![grupo("A", &[]), grupo("B", &["x"])]);
    assert!(grupos.grouped("x"));
    assert!(!grupos.grouped("y"));
    grupos.remove(1);
    assert!(!grupos.grouped("x"));
}

#[test]
fn apagar_tira_o_grupo_da_lista() {
    let mut grupos = Groups::new(vec![grupo("A", &["x"]), grupo("B", &[])]);
    assert_eq!(
        grupos.remove(0).map(|group| group.name),
        Some("A".to_owned())
    );
    assert_eq!(grupos.list().len(), 1);
    assert_eq!(grupos.remove(7), None);
}

#[test]
fn carregar_descarta_grupos_invalidos_e_apps_repetidos() {
    let grupos = Groups::new(vec![
        grupo("A", &["x", "x", "y"]),
        grupo("a", &["z"]),
        grupo(" ", &["w"]),
    ]);
    assert_eq!(grupos.list(), &[grupo("A", &["x", "y"])]);
}
