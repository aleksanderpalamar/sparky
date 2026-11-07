use crate::render::visual::{ansi, logo};

#[test]
fn o_logotipo_e_pequeno_e_sem_espacos_no_fim() {
    let linhas = logo::lines();
    assert!((9..=12).contains(&linhas.len()));
    assert!(logo::width() <= 24);
    for linha in &linhas {
        assert!(linha.width() > 0);
        assert_eq!(linha.clone().trimmed(), *linha);
    }
}

#[test]
fn a_largura_do_logotipo_e_a_da_linha_mais_larga() {
    let maior = logo::lines()
        .iter()
        .flat_map(|linha| linha.segments.iter())
        .map(|segmento| ansi::width(&segmento.text))
        .max();
    assert_eq!(maior, Some(logo::width()));
}
