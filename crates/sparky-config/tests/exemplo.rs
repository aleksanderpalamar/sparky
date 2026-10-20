use std::fs;
use std::path::PathBuf;

use sparky_config::{Config, parse};

#[test]
fn o_exemplo_distribuido_e_a_configuracao_padrao() {
    let arquivo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/config/sparky.toml");
    let contents = fs::read_to_string(&arquivo).unwrap_or_else(|erro| panic!("ler {}: {erro}", arquivo.display()));
    assert_eq!(parse(&contents), Ok(Config::default()));
}
