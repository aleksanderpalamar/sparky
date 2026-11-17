use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};

use super::raster::raster;

fn png(pasta: &Path, nome: &str, largura: u32, altura: u32) -> PathBuf {
    let caminho = pasta.join(nome);
    let imagem = RgbaImage::from_pixel(largura, altura, Rgba([200, 80, 40, 255]));
    assert!(imagem.save(&caminho).is_ok());
    caminho
}

#[test]
fn imagem_grande_e_reduzida_ao_tamanho_do_icone() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let Some(reduzida) = raster(&png(dir.path(), "chatgpt.png", 1024, 1024), 28) else {
        panic!("imagem válida");
    };
    assert_eq!((reduzida.width, reduzida.height), (28, 28));
    assert_eq!(reduzida.rgba.len(), 28 * 28 * 4);
    assert_eq!(&reduzida.rgba[..4], &[200, 80, 40, 255]);
}

#[test]
fn imagem_retangular_mantem_a_proporcao() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let Some(reduzida) = raster(&png(dir.path(), "faixa.png", 112, 56), 28) else {
        panic!("imagem válida");
    };
    assert_eq!((reduzida.width, reduzida.height), (28, 14));
}

#[test]
fn imagem_pequena_fica_no_tamanho_original() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let Some(original) = raster(&png(dir.path(), "mini.png", 16, 16), 28) else {
        panic!("imagem válida");
    };
    assert_eq!((original.width, original.height), (16, 16));
}

#[test]
fn arquivo_que_nao_e_imagem_nao_vira_icone() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let quebrado = dir.path().join("quebrado.png");
    assert!(std::fs::write(&quebrado, "png").is_ok());
    assert_eq!(raster(&quebrado, 28), None);
}
