use std::path::Path;

use super::thumbnail::supported;

#[test]
fn miniaturas_so_sao_pedidas_para_formatos_decodificaveis() {
    for name in ["foto.png", "FOTO.JPG", "imagem.jpeg", "capa.webp"] {
        assert!(supported(Path::new(name)), "{name}");
    }
    for name in ["vetor.svg", "animacao.gif", "imagem.avif", "arquivo.txt"] {
        assert!(!supported(Path::new(name)), "{name}");
    }
}
