use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::home_trash::HomeTrash;
use super::local_disk::LocalDisk;
use super::staged::{copy, rename_noreplace};
use crate::ports::Disk;

fn sobras(pasta: &Path) -> Vec<String> {
    fs::read_dir(pasta)
        .unwrap()
        .filter_map(Result::ok)
        .map(|item| item.file_name().to_string_lossy().into_owned())
        .filter(|nome| nome.starts_with(".sparky-files-"))
        .collect()
}

#[test]
fn copia_de_pasta_aparece_completa_e_sem_sobras() {
    let dir = tempfile::tempdir().unwrap();
    let origem = dir.path().join("Fotos");
    fs::create_dir_all(origem.join("2026")).unwrap();
    fs::write(origem.join("2026/praia.png"), "png").unwrap();
    fs::write(origem.join("lista.txt"), "a").unwrap();
    copy(&origem, &dir.path().join("Fotos (cópia)"), &mut |_| true).unwrap();
    let copiada = fs::read_to_string(dir.path().join("Fotos (cópia)/2026/praia.png")).unwrap();
    assert_eq!(copiada, "png");
    assert!(sobras(dir.path()).is_empty());
}

#[test]
fn copia_que_falha_no_meio_nao_deixa_nada_com_o_nome_final() {
    let dir = tempfile::tempdir().unwrap();
    let origem = dir.path().join("Documentos");
    fs::create_dir(&origem).unwrap();
    fs::write(origem.join("a.txt"), "a").unwrap();
    fs::write(origem.join("z-trancado.txt"), "segredo").unwrap();
    fs::set_permissions(
        origem.join("z-trancado.txt"),
        fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    if fs::read(origem.join("z-trancado.txt")).is_ok() {
        return;
    }
    let destino = dir.path().join("Documentos (cópia)");
    assert!(copy(&origem, &destino, &mut |_| true).is_err());
    assert!(!destino.exists());
    assert!(sobras(dir.path()).is_empty());
}

#[test]
fn copia_nao_sobrescreve_item_que_apareceu_no_destino() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("origem.txt"), "novo").unwrap();
    fs::write(dir.path().join("destino.txt"), "antigo").unwrap();
    let erro = copy(
        &dir.path().join("origem.txt"),
        &dir.path().join("destino.txt"),
        &mut |_| true,
    )
    .unwrap_err();
    assert_eq!(erro.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        fs::read_to_string(dir.path().join("destino.txt")).unwrap(),
        "antigo"
    );
    assert!(sobras(dir.path()).is_empty());
}

#[test]
fn renomear_nao_sobrescreve_item_existente() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();
    let erro = rename_noreplace(&dir.path().join("a.txt"), &dir.path().join("b.txt")).unwrap_err();
    assert_eq!(erro.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read_to_string(dir.path().join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(dir.path().join("b.txt")).unwrap(), "b");
}

#[test]
fn mover_pelo_disco_local_nao_sobrescreve_destino() {
    let dir = tempfile::tempdir().unwrap();
    let disco = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    fs::write(dir.path().join("nota.txt"), "nova").unwrap();
    fs::write(dir.path().join("final.txt"), "guardada").unwrap();
    assert!(
        disco
            .relocate(&dir.path().join("nota.txt"), &dir.path().join("final.txt"))
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("final.txt")).unwrap(),
        "guardada"
    );
    assert!(dir.path().join("nota.txt").exists());
}
