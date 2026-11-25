use std::os::unix::fs::PermissionsExt;

use super::storage::DiskStorage;
use crate::persistence::{Checkpoint, Kind, Knowledge, Restored};
use crate::ports::Storage;

fn mode(path: &std::path::Path) -> Option<u32> {
    std::fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode() & 0o777)
}

#[test]
fn grava_atomicamente_com_permissoes_restritas() {
    let Ok(root) = tempfile::tempdir() else {
        return;
    };
    let directory = root.path().join("sparky/resources");
    let Ok(storage) = DiskStorage::open(directory.clone()) else {
        panic!("diretório não criado")
    };
    assert!(storage.save("modelo.ckpt", b"abc").is_ok());
    assert_eq!(storage.load("modelo.ckpt").ok(), Some(b"abc".to_vec()));
    assert_eq!(mode(&directory), Some(0o700));
    assert_eq!(mode(&directory.join("modelo.ckpt")), Some(0o600));
    assert!(!directory.join("modelo.ckpt.new").exists());
}

#[test]
fn nomes_que_escapam_do_diretorio_sao_recusados() {
    let Ok(root) = tempfile::tempdir() else {
        return;
    };
    let Ok(storage) = DiskStorage::open(root.path().to_owned()) else {
        panic!("diretório não criado")
    };
    for name in ["../fora", "a/b", "", ".oculto"] {
        assert!(storage.save(name, b"x").is_err(), "{name}");
    }
}

#[test]
fn o_conhecimento_sobrevive_a_um_novo_processo() {
    let Ok(root) = tempfile::tempdir() else {
        return;
    };
    let model = Checkpoint {
        kind: Kind::Context,
        hardware: 3,
        values: vec![1.0, 2.0],
    };
    if let Ok(storage) = DiskStorage::open(root.path().to_owned()) {
        assert!(Knowledge::new(storage).store(&model).is_ok());
    }
    let Ok(storage) = DiskStorage::open(root.path().to_owned()) else {
        panic!("diretório não reaberto")
    };
    assert_eq!(
        Knowledge::new(storage).restore(Kind::Context, 3),
        Restored::Model(model)
    );
}

#[test]
fn rotacao_em_disco_deixa_atual_e_anterior_sem_temporario() {
    let Ok(root) = tempfile::tempdir() else {
        return;
    };
    let Ok(storage) = DiskStorage::open(root.path().to_owned()) else {
        panic!("diretório não criado")
    };
    let mut knowledge = Knowledge::new(storage);
    for value in [1.0, 2.0] {
        let model = Checkpoint {
            kind: Kind::Sequence,
            hardware: 3,
            values: vec![value],
        };
        assert!(knowledge.store(&model).is_ok());
    }
    let directory = root.path();
    assert!(directory.join("sequence.ckpt").exists());
    assert!(directory.join("sequence.previous.ckpt").exists());
    assert!(!directory.join("sequence.next.ckpt").exists());
    assert!(knowledge.roll_back(Kind::Sequence).is_ok());
    assert!(matches!(
        knowledge.restore(Kind::Sequence, 3),
        Restored::Model(model) if model.values == vec![1.0]
    ));
}
