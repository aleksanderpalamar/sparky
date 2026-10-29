use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use super::{FakeDisk, copia, recorte};
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::operations::{Failure, paste};

#[test]
fn copiar_para_a_mesma_pasta_ganha_nome_de_copia() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let pasted = paste(&copia(&["/h/foto.png"]), Path::new("/h"), &disk);
    assert_eq!(pasted.ok(), Some(vec!["foto (cópia).png".to_owned()]));
    assert_eq!(disk.calls(), ["copiar /h/foto.png /h/foto (cópia).png"]);
}

#[test]
fn recortar_para_a_mesma_pasta_nao_mexe_em_nada() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let pasted = paste(&recorte(&["/h/foto.png"]), Path::new("/h"), &disk);
    assert_eq!(pasted.ok(), Some(vec!["foto.png".to_owned()]));
    assert!(disk.calls().is_empty());
}

#[test]
fn recortar_para_outra_pasta_move_cada_item() {
    let disk = FakeDisk::with(&["/h/a.txt", "/h/b.txt"]);
    let pasted = paste(
        &recorte(&["/h/a.txt", "/h/b.txt"]),
        Path::new("/h/Docs"),
        &disk,
    );
    assert_eq!(
        pasted.ok(),
        Some(vec!["a.txt".to_owned(), "b.txt".to_owned()])
    );
    assert_eq!(
        disk.calls(),
        [
            "mover /h/a.txt /h/Docs/a.txt",
            "mover /h/b.txt /h/Docs/b.txt"
        ]
    );
}

#[test]
fn colar_uma_pasta_dentro_dela_mesma_e_recusado() {
    let disk = FakeDisk::with(&["/h/Fotos"]);
    let pasted = paste(&copia(&["/h/Fotos"]), Path::new("/h/Fotos/2026"), &disk);
    assert!(matches!(pasted, Err(Failure::IntoItself(name)) if name == "Fotos"));
    assert!(disk.calls().is_empty());
}

#[test]
fn caminho_com_ponto_ponto_nao_permite_copiar_pasta_para_si_mesma() {
    let disk = FakeDisk::with(&["/h/Fotos"]);
    let pasted = paste(&copia(&["/h/Fotos"]), Path::new("/h/../h/Fotos"), &disk);
    assert!(matches!(pasted, Err(Failure::IntoItself(name)) if name == "Fotos"));
    assert!(disk.calls().is_empty());
}

#[test]
fn link_para_origem_nao_permite_copia_recursiva() {
    let dir = tempfile::tempdir().unwrap();
    let photos = dir.path().join("Fotos");
    let alias = dir.path().join("alias");
    fs::create_dir(&photos).unwrap();
    symlink(&photos, &alias).unwrap();
    let disk = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    let pasted = paste(
        &crate::domain::clip::Clip::Copy(vec![photos.clone()]),
        &alias,
        &disk,
    );
    assert!(matches!(pasted, Err(Failure::IntoItself(name)) if name == "Fotos"));
    assert!(fs::read_dir(&photos).unwrap().next().is_none());
}

#[test]
fn copia_link_quebrado_sem_resolver_o_alvo() {
    let dir = tempfile::tempdir().unwrap();
    let link = dir.path().join("atalho");
    symlink("ausente", &link).unwrap();
    let disk = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    let pasted = paste(
        &crate::domain::clip::Clip::Copy(vec![link]),
        dir.path(),
        &disk,
    );
    assert_eq!(pasted.ok(), Some(vec!["atalho (cópia)".to_owned()]));
    assert_eq!(
        fs::read_link(dir.path().join("atalho (cópia)")).ok(),
        Some("ausente".into())
    );
}
