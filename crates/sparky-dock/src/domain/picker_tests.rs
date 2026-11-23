use super::picker::Picker;
use super::pinned::Pinned;
use super::support::{app, ids};

fn catalog() -> Vec<super::app::App> {
    vec![
        app("firefox", "Firefox"),
        app("steam", "Steam"),
        app("sparky-files", "Arquivos"),
        app("libreoffice-writer", "LibreOffice Writer"),
    ]
}

fn names(picker: &Picker, apps: &[super::app::App]) -> Vec<String> {
    picker
        .visible(apps)
        .iter()
        .map(|app| app.name.clone())
        .collect()
}

#[test]
fn a_busca_filtra_pelo_nome_sem_diferenciar_maiusculas_e_acentos() {
    let apps = catalog();
    let mut picker = Picker::default();
    assert_eq!(names(&picker, &apps).len(), 4);
    picker.search("  STE ".to_owned());
    assert_eq!(names(&picker, &apps), ids(&["Steam"]));
    picker.search("arquivós".to_owned());
    assert_eq!(names(&picker, &apps), ids(&["Arquivos"]));
}

#[test]
fn marca_e_desmarca_varios_na_ordem_escolhida() {
    let pinned = Pinned::default();
    let mut picker = Picker::default();
    picker.toggle("steam", &pinned, usize::MAX);
    picker.toggle("firefox", &pinned, usize::MAX);
    picker.toggle("libreoffice-writer", &pinned, usize::MAX);
    picker.toggle("firefox", &pinned, usize::MAX);
    assert_eq!(picker.chosen(), ids(&["steam", "libreoffice-writer"]));
}

#[test]
fn a_selecao_sobrevive_a_troca_de_filtro() {
    let apps = catalog();
    let pinned = Pinned::default();
    let mut picker = Picker::default();
    picker.search("fire".to_owned());
    picker.toggle("firefox", &pinned, usize::MAX);
    picker.search("steam".to_owned());
    picker.toggle("steam", &pinned, usize::MAX);
    picker.search(String::new());
    assert!(picker.is_chosen("firefox") && picker.is_chosen("steam"));
    assert_eq!(names(&picker, &apps).len(), 4);
}

#[test]
fn aplicativo_ja_fixado_nao_entra_na_selecao() {
    let pinned = Pinned::default();
    let mut picker = Picker::default();
    picker.toggle("sparky-files", &pinned, usize::MAX);
    assert!(picker.chosen().is_empty());
}

#[test]
fn sem_espaco_no_dock_nao_marca_mais_mas_ainda_desmarca() {
    let pinned = Pinned::default();
    let mut picker = Picker::default();
    picker.toggle("steam", &pinned, 1);
    picker.toggle("firefox", &pinned, 1);
    assert_eq!(picker.chosen(), ids(&["steam"]));
    assert!(picker.full(1));
    picker.toggle("steam", &pinned, 1);
    assert!(picker.chosen().is_empty());
    assert!(!picker.full(1));
}
