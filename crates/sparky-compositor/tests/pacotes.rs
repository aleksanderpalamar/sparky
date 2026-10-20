use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ler(caminho: &Path) -> String {
    fs::read_to_string(caminho).unwrap_or_else(|erro| panic!("ler {}: {erro}", caminho.display()))
}

fn manifesto(caminho: &Path) -> Value {
    ler(caminho).parse().unwrap_or_else(|erro| panic!("{}: {erro}", caminho.display()))
}

fn metadados(formato: &str) -> Value {
    let manifesto = manifesto(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    manifesto["package"]["metadata"][formato].clone()
}

fn origens(formato: &str) -> Vec<String> {
    let assets = metadados(formato)["assets"].as_array().cloned().unwrap_or_default();
    assets
        .iter()
        .filter_map(|asset| match asset {
            Value::Array(campos) => campos.first().and_then(Value::as_str).map(str::to_owned),
            Value::Table(campos) => campos.get("source").and_then(Value::as_str).map(str::to_owned),
            _ => None,
        })
        .collect()
}

fn binarios_do(formato: &str) -> BTreeSet<String> {
    origens(formato)
        .iter()
        .filter_map(|origem| origem.strip_prefix("target/release/").map(str::to_owned))
        .collect()
}

fn dados_do(formato: &str) -> BTreeSet<String> {
    origens(formato)
        .iter()
        .filter_map(|origem| origem.strip_prefix("../../data/").map(str::to_owned))
        .collect()
}

fn binarios_do_workspace() -> BTreeSet<String> {
    let mut nomes = BTreeSet::new();
    for crate_dir in fs::read_dir(raiz().join("crates")).into_iter().flatten().flatten() {
        let manifesto = manifesto(&crate_dir.path().join("Cargo.toml"));
        let declarados = manifesto.get("bin").and_then(Value::as_array).cloned().unwrap_or_default();
        nomes.extend(declarados.iter().filter_map(|bin| bin.get("name")?.as_str().map(str::to_owned)));
        let soltos = fs::read_dir(crate_dir.path().join("src/bin")).into_iter().flatten().flatten();
        nomes.extend(soltos.filter_map(|arquivo| Some(arquivo.path().file_stem()?.to_str()?.to_owned())));
    }
    nomes
}

fn arquivos_de_dados(pasta: &Path, base: &Path, achados: &mut BTreeSet<String>) {
    for item in fs::read_dir(pasta).into_iter().flatten().flatten() {
        let caminho = item.path();
        match caminho.is_dir() {
            true => arquivos_de_dados(&caminho, base, achados),
            false => {
                if let Ok(relativo) = caminho.strip_prefix(base) {
                    achados.insert(relativo.to_string_lossy().into_owned());
                }
            }
        }
    }
}

fn pkgbuild() -> String {
    ler(&raiz().join("packaging/arch/PKGBUILD"))
}

fn campo(texto: &str, chave: &str) -> Option<String> {
    let marca = format!("{chave}=");
    texto.lines().find_map(|linha| linha.strip_prefix(&marca)).map(|valor| valor.trim_matches(['\'', '(', ')']).to_owned())
}

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
        .find_map(|linha| linha.trim().strip_prefix("for binary in ").map(str::to_owned))
        .unwrap_or_default();
    let instalados: BTreeSet<String> = linha.trim_end_matches("; do").split_whitespace().map(str::to_owned).collect();
    assert_eq!(instalados, binarios_do_workspace());
}

#[test]
fn todo_arquivo_de_data_vai_no_deb_e_no_rpm() {
    let base = raiz().join("data");
    let mut presentes = BTreeSet::new();
    arquivos_de_dados(&base, &base, &mut presentes);
    assert_eq!(dados_do("deb"), presentes);
    assert_eq!(dados_do("generate-rpm"), presentes);
}

#[test]
fn o_pkgbuild_instala_todo_arquivo_de_data() {
    let base = raiz().join("data");
    let mut presentes = BTreeSet::new();
    arquivos_de_dados(&base, &base, &mut presentes);
    let texto = pkgbuild();
    for arquivo in presentes {
        let pasta = Path::new(&arquivo).parent().map(|pasta| format!("data/{}/*", pasta.display()));
        let citado = texto.contains(&format!("data/{arquivo}")) || pasta.is_some_and(|pasta| texto.contains(&pasta));
        assert!(citado, "o PKGBUILD não instala data/{arquivo}");
    }
}

#[test]
fn as_origens_que_nao_sao_binarios_existem() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for origem in origens("deb").into_iter().chain(origens("generate-rpm")) {
        if !origem.starts_with("target/") {
            assert!(crate_dir.join(&origem).is_file(), "{origem} não existe");
        }
    }
}

#[test]
fn versao_e_licenca_batem_nos_tres_formatos() {
    let workspace = manifesto(&raiz().join("Cargo.toml"));
    let pacote = &workspace["workspace"]["package"];
    let texto = pkgbuild();
    assert_eq!(campo(&texto, "pkgver").as_deref(), pacote["version"].as_str());
    assert_eq!(campo(&texto, "license").as_deref(), pacote["license"].as_str());
    assert_eq!(metadados("generate-rpm")["license"].as_str(), pacote["license"].as_str());
    assert!(raiz().join("LICENSE").is_file());
}
