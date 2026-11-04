use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

pub(super) fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn ler(caminho: &Path) -> String {
    fs::read_to_string(caminho).unwrap_or_else(|erro| panic!("ler {}: {erro}", caminho.display()))
}

pub(super) fn manifesto(caminho: &Path) -> Value {
    ler(caminho)
        .parse()
        .unwrap_or_else(|erro| panic!("{}: {erro}", caminho.display()))
}

pub(super) fn metadados(formato: &str) -> Value {
    let manifesto = manifesto(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    manifesto["package"]["metadata"][formato].clone()
}

pub(super) fn variante(formato: &str) -> Value {
    metadados(formato)["variants"]["wallpapers"].clone()
}

pub(super) fn origens(formato: &str) -> Vec<String> {
    origens_de(&metadados(formato))
}

pub(super) fn origens_de(metadados: &Value) -> Vec<String> {
    let assets = metadados
        .get("assets")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assets
        .iter()
        .filter_map(|asset| match asset {
            Value::Array(campos) => campos.first().and_then(Value::as_str).map(str::to_owned),
            Value::Table(campos) => campos
                .get("source")
                .and_then(Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
        .collect()
}

pub(super) fn binarios_do(formato: &str) -> BTreeSet<String> {
    origens(formato)
        .iter()
        .filter_map(|origem| origem.strip_prefix("target/release/").map(str::to_owned))
        .collect()
}

pub(super) fn expandir(origem: &str) -> Vec<String> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut caminhos = vec![PathBuf::new()];
    for parte in origem.split('/') {
        caminhos = caminhos
            .into_iter()
            .flat_map(|caminho| match parte {
                "*" | "*.ftl" => fs::read_dir(base.join(&caminho))
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|item| {
                        parte == "*" || item.path().extension().is_some_and(|ext| ext == "ftl")
                    })
                    .map(|item| caminho.join(item.file_name()))
                    .collect(),
                _ => vec![caminho.join(parte)],
            })
            .collect();
    }
    caminhos
        .into_iter()
        .filter(|caminho| base.join(caminho).is_file())
        .map(|caminho| caminho.to_string_lossy().into_owned())
        .collect()
}

pub(super) fn dados_do(formato: &str) -> BTreeSet<String> {
    dados_de(origens(formato))
}

pub(super) fn dados_de(origens: Vec<String>) -> BTreeSet<String> {
    origens
        .iter()
        .flat_map(|origem| expandir(origem))
        .filter_map(|origem| origem.strip_prefix("../../data/").map(str::to_owned))
        .collect()
}

pub(super) fn padroes_que_cobrem(arquivo: &str) -> Vec<String> {
    let partes: Vec<&str> = arquivo.split('/').collect();
    (1..partes.len())
        .map(|fixas| {
            format!(
                "data/{}/{}*",
                partes[..fixas].join("/"),
                "*/".repeat(partes.len() - fixas - 1)
            )
        })
        .collect()
}

pub(super) fn binarios_do_workspace() -> BTreeSet<String> {
    let mut nomes = BTreeSet::new();
    for crate_dir in fs::read_dir(raiz().join("crates"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let manifesto = manifesto(&crate_dir.path().join("Cargo.toml"));
        let declarados = manifesto
            .get("bin")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        nomes.extend(
            declarados
                .iter()
                .filter_map(|bin| bin.get("name")?.as_str().map(str::to_owned)),
        );
        let soltos = fs::read_dir(crate_dir.path().join("src/bin"))
            .into_iter()
            .flatten()
            .flatten();
        nomes.extend(
            soltos.filter_map(|arquivo| Some(arquivo.path().file_stem()?.to_str()?.to_owned())),
        );
    }
    nomes
}

pub(super) fn arquivos_de_dados(pasta: &Path, base: &Path, achados: &mut BTreeSet<String>) {
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

pub(super) fn pkgbuild() -> String {
    ler(&raiz().join("packaging/arch/PKGBUILD"))
}

pub(super) fn campo(texto: &str, chave: &str) -> Option<String> {
    let marca = format!("{chave}=");
    texto
        .lines()
        .find_map(|linha| linha.strip_prefix(&marca))
        .map(|valor| valor.trim_matches(['\'', '(', ')']).to_owned())
}
