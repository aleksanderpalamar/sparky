use std::path::{Path, PathBuf};

use super::relocation::relocate;

fn mapa(pares: &[(&str, &str)]) -> Vec<(PathBuf, PathBuf)> {
    pares
        .iter()
        .map(|(de, para)| (PathBuf::from(de), PathBuf::from(para)))
        .collect()
}

#[test]
fn renomeacoes_em_cadeia_se_aplicam_ao_mesmo_tempo() {
    let moves = mapa(&[("/p/a", "/p/aa"), ("/p/aa", "/p/aaaa")]);
    assert_eq!(relocate(Path::new("/p/a"), &moves), PathBuf::from("/p/aa"));
    assert_eq!(
        relocate(Path::new("/p/aa"), &moves),
        PathBuf::from("/p/aaaa")
    );
}

#[test]
fn trocas_entre_dois_nomes_nao_se_anulam() {
    let moves = mapa(&[("/p/x", "/p/y"), ("/p/y", "/p/x")]);
    assert_eq!(
        relocate(Path::new("/p/x/f.txt"), &moves),
        PathBuf::from("/p/y/f.txt")
    );
    assert_eq!(relocate(Path::new("/p/y"), &moves), PathBuf::from("/p/x"));
}

#[test]
fn filhos_seguem_a_pasta_mais_especifica() {
    let moves = mapa(&[("/p", "/q"), ("/p/sub", "/r")]);
    assert_eq!(
        relocate(Path::new("/p/sub/f"), &moves),
        PathBuf::from("/r/f")
    );
    assert_eq!(relocate(Path::new("/p/g"), &moves), PathBuf::from("/q/g"));
}

#[test]
fn caminhos_fora_das_renomeacoes_ficam_iguais() {
    let moves = mapa(&[("/p/a", "/p/b")]);
    assert_eq!(relocate(Path::new("/p/ab"), &moves), PathBuf::from("/p/ab"));
    assert_eq!(
        relocate(Path::new("/outro"), &moves),
        PathBuf::from("/outro")
    );
}
