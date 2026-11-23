use crate::parse;

#[test]
fn frosted_glass_vem_desligado_e_e_lido_de_general() {
    let config = parse("").unwrap_or_default();
    assert!(!config.general.frosted_glass);
    let Ok(ligado) = parse("[general]\nfrosted_glass = true\n") else {
        panic!("configuração válida")
    };
    assert!(ligado.general.frosted_glass);
}
