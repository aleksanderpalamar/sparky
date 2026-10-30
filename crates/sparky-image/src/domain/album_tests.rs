use std::path::{Path, PathBuf};

use super::album::{Album, Edge, Step};

fn caminhos(nomes: &[&str]) -> Vec<PathBuf> {
    nomes
        .iter()
        .map(|nome| Path::new("/fotos").join(nome))
        .collect()
}

#[test]
fn ordena_como_o_gerenciador_de_arquivos_e_comeca_na_imagem_aberta() {
    let mut album = Album::new(
        caminhos(&["b.png", "A.jpg", "c.webp"]),
        Path::new("/fotos/b.png"),
    );
    assert_eq!(album.current(), Path::new("/fotos/b.png"));
    assert_eq!(album.position(), (2, 3));
    album.jump(Edge::First);
    assert_eq!(album.current(), Path::new("/fotos/A.jpg"));
}

#[test]
fn avancar_e_voltar_dao_a_volta_nas_pontas() {
    let mut album = Album::new(
        caminhos(&["a.png", "b.png", "c.png"]),
        Path::new("/fotos/c.png"),
    );
    album.step(Step::Next);
    assert_eq!(album.current(), Path::new("/fotos/a.png"));
    album.step(Step::Previous);
    assert_eq!(album.current(), Path::new("/fotos/c.png"));
    album.jump(Edge::First);
    assert_eq!(album.position(), (1, 3));
    album.jump(Edge::Last);
    assert_eq!(album.position(), (3, 3));
}

#[test]
fn imagem_aberta_fora_da_lista_entra_na_posicao_ordenada() {
    let album = Album::new(caminhos(&["a.png", "c.png"]), Path::new("/fotos/b.foto"));
    assert_eq!(album.current(), Path::new("/fotos/b.foto"));
    assert_eq!(album.position(), (2, 3));
}

#[test]
fn imagem_sozinha_nao_muda_ao_navegar() {
    let mut album = Album::new(Vec::new(), Path::new("/fotos/unica.png"));
    album.step(Step::Next);
    assert_eq!(album.current(), Path::new("/fotos/unica.png"));
    assert_eq!(album.position(), (1, 1));
}

#[test]
fn nomes_repetidos_nao_duplicam_a_imagem_aberta() {
    let album = Album::new(caminhos(&["a.png", "a.png"]), Path::new("/fotos/a.png"));
    assert_eq!(album.position(), (1, 1));
}
