use super::pinned::{NATIVE, Pinned};
use super::support::ids;

#[test]
fn comeca_com_os_aplicativos_nativos_na_ordem_pedida() {
    assert_eq!(
        Pinned::default().ids(),
        ids(&[
            "sparky-settings",
            "sparky-files",
            "sparky-terminal",
            "sparky-store"
        ])
    );
    assert_eq!(NATIVE.len(), 4);
}

#[test]
fn adicionar_um_aplicativo_vai_para_o_fim() {
    let mut pinned = Pinned::default();
    assert!(pinned.add(ids(&["firefox"])));
    assert_eq!(pinned.ids().last().map(String::as_str), Some("firefox"));
}

#[test]
fn adicionar_varios_mantem_a_ordem_da_selecao() {
    let mut pinned = Pinned::from_ids(ids(&["a"]));
    assert!(pinned.add(ids(&["c", "b"])));
    assert_eq!(pinned.ids(), ids(&["a", "c", "b"]));
}

#[test]
fn nao_repete_aplicativo_ja_fixado() {
    let mut pinned = Pinned::default();
    assert!(!pinned.add(ids(&["sparky-files"])));
    assert!(pinned.add(ids(&["steam", "steam"])));
    assert_eq!(pinned.ids().iter().filter(|id| *id == "steam").count(), 1);
}

#[test]
fn ignora_entradas_invalidas_e_duplicadas_ao_restaurar() {
    let pinned = Pinned::from_ids(ids(&["", " firefox ", "a/b", "firefox", "linha\u{7}"]));
    assert_eq!(pinned.ids(), ids(&["firefox"]));
}

#[test]
fn aplicativo_desinstalado_some_do_dock_sem_sair_da_configuracao() {
    let apps = vec![
        super::support::app("sparky-store", "Loja"),
        super::support::app("sparky-files", "Arquivos"),
    ];
    let pinned = Pinned::default();
    let shown: Vec<&str> = pinned
        .resolve(&apps)
        .iter()
        .map(|app| app.id.as_str())
        .collect();
    assert_eq!(shown, ["sparky-files", "sparky-store"]);
    assert_eq!(pinned.ids().len(), 4);
}

#[test]
fn remover_tira_so_o_aplicativo_pedido_e_mantem_a_ordem() {
    let mut pinned = Pinned::default();
    assert!(pinned.remove("sparky-files"));
    assert_eq!(
        pinned.ids(),
        ids(&["sparky-settings", "sparky-terminal", "sparky-store"])
    );
    assert!(!pinned.remove("sparky-files"));
}

#[test]
fn reposicionar_poe_antes_do_vizinho_ou_no_fim() {
    let mut pinned = Pinned::from_ids(ids(&["a", "b", "c"]));
    assert!(pinned.place("c", Some("a")));
    assert_eq!(pinned.ids(), ids(&["c", "a", "b"]));
    assert!(pinned.place("c", None));
    assert_eq!(pinned.ids(), ids(&["a", "b", "c"]));
    assert!(!pinned.place("z", None));
    assert!(!pinned.place("c", None));
}
