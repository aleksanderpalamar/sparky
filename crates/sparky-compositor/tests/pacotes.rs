#[path = "pacotes/support.rs"]
mod support;

use std::collections::BTreeSet;
use std::path::PathBuf;

use support::*;

#[test]
fn deb_e_rpm_levam_todos_os_binarios_do_workspace() {
    let esperados = binarios_do_workspace();
    assert_eq!(binarios_do("deb"), esperados);
    assert_eq!(binarios_do("generate-rpm"), esperados);
}

#[test]
fn o_pkgbuild_instala_os_mesmos_binarios() {
    let linha = pkgbuild()
        .lines()
        .find_map(|linha| {
            linha
                .trim()
                .strip_prefix("for binary in ")
                .map(str::to_owned)
        })
        .unwrap_or_default();
    let instalados: BTreeSet<String> = linha
        .trim_end_matches("; do")
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    assert_eq!(instalados, binarios_do_workspace());
}

fn dados_presentes() -> (BTreeSet<String>, BTreeSet<String>) {
    let base = raiz().join("data");
    let mut presentes = BTreeSet::new();
    arquivos_de_dados(&base, &base, &mut presentes);
    presentes
        .into_iter()
        .partition(|arquivo| !arquivo.starts_with("wallpapers/"))
}

#[test]
fn todo_arquivo_de_data_vai_no_deb_e_no_rpm() {
    let (sistema, _) = dados_presentes();
    assert_eq!(dados_do("deb"), sistema);
    assert_eq!(dados_do("generate-rpm"), sistema);
}

#[test]
fn os_papeis_de_parede_vao_num_pacote_proprio() {
    let (_, papeis) = dados_presentes();
    assert!(papeis.len() > 20);
    for formato in ["deb", "generate-rpm"] {
        let pacote = variante(formato);
        assert_eq!(
            pacote["name"].as_str(),
            Some("sparky-wallpapers"),
            "{formato}"
        );
        assert_eq!(dados_de(origens_de(&pacote)), papeis, "{formato}");
    }
    let texto = pkgbuild();
    let funcao = texto
        .split("package_sparky-wallpapers()")
        .nth(1)
        .unwrap_or_default();
    assert!(texto.contains("pkgname=(sparky sparky-wallpapers)"));
    assert!(funcao.contains("data/wallpapers/*/*"));
    assert!(
        !texto
            .split("package_sparky-wallpapers()")
            .next()
            .unwrap_or_default()
            .contains("data/wallpapers")
    );
}

#[test]
fn o_pkgbuild_instala_todo_arquivo_de_data() {
    let base = raiz().join("data");
    let mut presentes = BTreeSet::new();
    arquivos_de_dados(&base, &base, &mut presentes);
    let texto = pkgbuild();
    for arquivo in presentes {
        let citado = texto.contains(&format!("data/{arquivo}"))
            || padroes_que_cobrem(&arquivo)
                .iter()
                .any(|padrao| texto.contains(padrao.as_str()));
        assert!(citado, "o PKGBUILD não instala data/{arquivo}");
    }
}

#[test]
fn as_origens_que_nao_sao_binarios_existem() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for origem in origens("deb").into_iter().chain(origens("generate-rpm")) {
        if origem.contains('*') {
            assert!(
                !expandir(&origem).is_empty(),
                "{origem} não acha nenhum arquivo"
            );
        } else if !origem.starts_with("target/") {
            assert!(crate_dir.join(&origem).is_file(), "{origem} não existe");
        }
    }
}

#[test]
fn versao_e_licenca_batem_nos_tres_formatos() {
    let workspace = manifesto(&raiz().join("Cargo.toml"));
    let pacote = &workspace["workspace"]["package"];
    let texto = pkgbuild();
    assert_eq!(
        campo(&texto, "pkgver").as_deref(),
        pacote["version"].as_str()
    );
    assert_eq!(
        campo(&texto, "license").as_deref(),
        pacote["license"].as_str()
    );
    assert_eq!(
        metadados("generate-rpm")["license"].as_str(),
        pacote["license"].as_str()
    );
    assert!(raiz().join("LICENSE").is_file());
}
