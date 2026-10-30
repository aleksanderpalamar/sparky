use std::fs;

use super::gallery::FolderGallery;
use crate::ports::Gallery;

#[test]
fn lista_somente_arquivos_de_imagem_da_pasta() {
    let dir = tempfile::tempdir().unwrap();
    for nome in ["a.png", "B.JPG", "notas.txt", "c.webp"] {
        fs::write(dir.path().join(nome), b"x").unwrap();
    }
    fs::create_dir(dir.path().join("pasta.png")).unwrap();
    let mut nomes: Vec<_> = FolderGallery
        .images(dir.path())
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect();
    nomes.sort();
    assert_eq!(nomes, ["B.JPG", "a.png", "c.webp"]);
}

#[test]
fn pasta_inexistente_resulta_em_lista_vazia() {
    let dir = tempfile::tempdir().unwrap();
    assert!(FolderGallery.images(&dir.path().join("sumiu")).is_empty());
}
