use crate::domain::app::{App, Runs};
use crate::domain::search::rank;

fn app(name: &str, keywords: &[&str]) -> App {
    App {
        id: name.to_lowercase(),
        name: name.to_owned(),
        detail: None,
        keywords: keywords.iter().map(|word| (*word).to_owned()).collect(),
        command: name.to_lowercase(),
        runs: Runs::Directly,
    }
}

fn catalogo() -> Vec<App> {
    vec![
        app("Firefox", &["navegador", "web"]),
        app("Arquivos", &["pastas"]),
        app("Configurações do Sparky", &["preferências"]),
        app("Fonts", &[]),
    ]
}

fn nomes(apps: &[App], query: &str) -> Vec<String> {
    rank(apps, query).into_iter().map(|found| found.name.clone()).collect()
}

#[test]
fn sem_busca_lista_tudo_em_ordem_alfabetica() {
    assert_eq!(nomes(&catalogo(), "  "), ["Arquivos", "Configurações do Sparky", "Firefox", "Fonts"]);
}

#[test]
fn o_comeco_do_nome_vem_antes_do_meio_e_da_palavra_chave() {
    assert_eq!(nomes(&catalogo(), "f"), ["Firefox", "Fonts", "Configurações do Sparky"]);
}

#[test]
fn comeco_de_palavra_e_palavra_chave_tambem_acham() {
    assert_eq!(nomes(&catalogo(), "spar"), ["Configurações do Sparky"]);
    assert_eq!(nomes(&catalogo(), "NAVEG"), ["Firefox"]);
}

#[test]
fn letras_na_ordem_acham_mesmo_separadas() {
    assert_eq!(nomes(&catalogo(), "ffx"), ["Firefox"]);
}

#[test]
fn busca_sem_resultado_fica_vazia() {
    assert!(nomes(&catalogo(), "xyz").is_empty());
}
