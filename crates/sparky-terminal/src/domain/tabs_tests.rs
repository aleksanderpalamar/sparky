use super::tabs::Tabs;

#[test]
fn uma_aba_nova_recebe_identidade_propria_e_o_foco() {
    let mut tabs = Tabs::new("primeira");
    let first = tabs.active_id();
    let second = tabs.insert("segunda");
    assert_ne!(second, first);
    assert_eq!(tabs.active().map(|tab| tab.value), Some("segunda"));
    assert_eq!(tabs.iter().count(), 2);
}

#[test]
fn alternar_abas_preserva_o_conteudo_e_volta_pelas_extremidades() {
    let mut tabs = Tabs::new(String::from("primeira"));
    let first = tabs.active_id();
    let second = tabs.insert(String::from("segunda"));
    tabs.cycle(1);
    assert_eq!(tabs.active_id(), first);
    tabs.cycle(-1);
    assert_eq!(tabs.active_id(), second);
    assert_eq!(tabs.active().map(|tab| tab.value.as_str()), Some("segunda"));
}

#[test]
fn fechar_a_ativa_escolhe_a_vizinha_sem_reutilizar_identidades() {
    let mut tabs = Tabs::new("primeira");
    let Some(first) = tabs.active_id() else {
        panic!("sem aba inicial")
    };
    let Some(second) = tabs.insert("segunda") else {
        panic!("sem segunda aba")
    };
    let third = tabs.insert("terceira");
    tabs.select(second);
    assert_eq!(tabs.remove(second), Some("segunda"));
    assert_eq!(tabs.active_id(), third);
    tabs.remove(first);
    let fourth = tabs.insert("quarta");
    assert_ne!(fourth, Some(second));
    assert!(!tabs.select(second));
}

#[test]
fn fechar_uma_aba_inativa_preserva_o_foco() {
    let mut tabs = Tabs::new("primeira");
    let Some(first) = tabs.active_id() else {
        panic!("sem aba inicial")
    };
    let second = tabs.insert("segunda");
    tabs.remove(first);
    assert_eq!(tabs.active_id(), second);
    assert!(tabs.remove(first).is_none());
}

#[test]
fn fechar_a_ultima_deixa_a_colecao_vazia() {
    let mut tabs = Tabs::new("primeira");
    let Some(id) = tabs.active_id() else {
        panic!("sem aba inicial")
    };
    tabs.remove(id);
    tabs.cycle(1);
    assert!(tabs.is_empty());
    assert!(tabs.active().is_none());
}
