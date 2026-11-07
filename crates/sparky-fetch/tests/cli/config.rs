use std::fs;
use std::path::Path;

use super::run::{fetch, json};

fn configurar(home: &Path, texto: &str) {
    let pasta = home.join("config/sparky-fetch");
    fs::create_dir_all(&pasta).unwrap_or_else(|error| panic!("{error}"));
    fs::write(pasta.join("config.toml"), texto).unwrap_or_else(|error| panic!("{error}"));
}

fn campos(saida: &serde_json::Value) -> Vec<String> {
    saida["entries"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry["field"].as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn o_toml_escolhe_campos_e_formato_e_a_linha_de_comando_vence() {
    let home = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    configurar(
        home.path(),
        "fields = [\"kernel\", \"os\"]\nformat = \"json\"\n",
    );
    let do_arquivo = fetch(&[], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(campos(&json(&do_arquivo)), ["kernel", "os"]);
    let pela_linha = fetch(&["--fields", "os"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(campos(&json(&pela_linha)), ["os"]);
    let texto = fetch(&["--plain"], home.path(), None, "pt_BR.UTF-8");
    assert!(texto.stdout.starts_with("Kernel: "), "{}", texto.stdout);
}

#[test]
fn opcao_invalida_sai_com_2_e_mostra_o_uso() {
    let home = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let run = fetch(&["--voar"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(run.code, Some(2));
    assert!(
        run.stderr.contains("--voar") && run.stderr.contains("Uso:"),
        "{}",
        run.stderr
    );
}

#[test]
fn configuracao_invalida_sai_com_1_e_diz_o_arquivo() {
    let home = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    configurar(home.path(), "format = \"html\"\n");
    let run = fetch(&[], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(run.code, Some(1));
    assert!(
        run.stderr.contains("sparky-fetch/config.toml"),
        "{}",
        run.stderr
    );
}

#[test]
fn versao_e_lista_de_campos() {
    let home = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let versao = fetch(&["--version"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(
        versao.stdout,
        format!("sparky-fetch {}\n", env!("CARGO_PKG_VERSION"))
    );
    let lista = fetch(&["--list-fields"], home.path(), None, "pt_BR.UTF-8");
    assert_eq!(lista.stdout.lines().count(), 18);
}
