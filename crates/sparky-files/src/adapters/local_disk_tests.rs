use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use tempfile::TempDir;

use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::adapters::transfer::LocalTransfer;
use crate::domain::entry::{Kind, Meta};
use crate::domain::transfer::{Action, Step};
use crate::ports::{Disk, Transfer};

fn ambiente() -> Option<(TempDir, LocalDisk)> {
    let dir = tempfile::tempdir().ok()?;
    let disk = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    Some((dir, disk))
}

fn escrever(path: &Path, texto: &str) {
    if let Err(error) = fs::write(path, texto) {
        panic!("não consegui preparar {}: {error}", path.display());
    }
}

#[test]
fn lista_pastas_arquivos_e_links_quebrados() {
    let Some((dir, disk)) = ambiente() else {
        panic!("sem pasta temporária")
    };
    escrever(&dir.path().join("a.txt"), "12345");
    assert!(fs::create_dir(dir.path().join("Fotos")).is_ok());
    assert!(symlink(dir.path().join("sumiu"), dir.path().join("quebrado")).is_ok());
    let mut entries = disk.list(dir.path()).unwrap_or_default();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let resumo: Vec<(&str, Kind)> = entries.iter().map(|e| (e.name.as_str(), e.kind)).collect();
    assert_eq!(
        resumo,
        [
            ("Fotos", Kind::Directory),
            ("a.txt", Kind::File),
            ("quebrado", Kind::File)
        ]
    );
    assert!(entries.iter().all(|entry| entry.meta == Meta::Pending));
    assert!(disk.exists(&dir.path().join("quebrado")));
    let detalhe = disk.detail(&dir.path().join("a.txt"));
    assert!(matches!(
        detalhe,
        Meta::Known {
            size: 5,
            modified: Some(_)
        }
    ));
    assert!(matches!(
        disk.detail(&dir.path().join("quebrado")),
        Meta::Known { .. }
    ));
}

#[test]
fn copia_a_pasta_inteira_com_os_links() {
    let Some((dir, _disk)) = ambiente() else {
        panic!("sem pasta temporária")
    };
    let origem = dir.path().join("origem");
    assert!(fs::create_dir_all(origem.join("sub")).is_ok());
    escrever(&origem.join("sub/nota.txt"), "oi");
    assert!(symlink("sub/nota.txt", origem.join("atalho")).is_ok());
    let transfer = LocalTransfer::new(HomeTrash::new(dir.path().join("Trash")), None);
    let passo = Step {
        source: origem.clone(),
        target: dir.path().join("destino"),
        action: Action::Copy,
        replace: false,
    };
    assert!(transfer.run(&passo, &mut |_| true).is_ok());
    assert_eq!(
        fs::read_to_string(dir.path().join("destino/sub/nota.txt"))
            .ok()
            .as_deref(),
        Some("oi")
    );
    assert_eq!(
        fs::read_link(dir.path().join("destino/atalho")).ok(),
        Some("sub/nota.txt".into())
    );
    assert!(origem.exists());
}

#[test]
fn mover_troca_o_lugar_e_criar_arquivo_nao_sobrescreve() {
    let Some((dir, disk)) = ambiente() else {
        panic!("sem pasta temporária")
    };
    escrever(&dir.path().join("a.txt"), "conteúdo");
    assert!(
        disk.relocate(&dir.path().join("a.txt"), &dir.path().join("b.txt"))
            .is_ok()
    );
    assert!(!dir.path().join("a.txt").exists());
    assert!(disk.create_file(&dir.path().join("b.txt")).is_err());
    assert_eq!(
        fs::read_to_string(dir.path().join("b.txt")).ok().as_deref(),
        Some("conteúdo")
    );
    assert!(disk.create_dir(&dir.path().join("Nova")).is_ok());
    assert!(disk.stamp(dir.path()).is_some());
}

#[test]
fn a_lixeira_guarda_o_item_e_o_registro_sem_sobrescrever() {
    let Some((dir, disk)) = ambiente() else {
        panic!("sem pasta temporária")
    };
    for _ in 0..2 {
        escrever(&dir.path().join("nota.txt"), "x");
        assert!(disk.trash(&dir.path().join("nota.txt")).is_ok());
    }
    let trash = dir.path().join("Trash");
    assert!(trash.join("files/nota.txt").exists());
    assert!(trash.join("files/nota (cópia).txt").exists());
    let registro = fs::read_to_string(trash.join("info/nota.txt.trashinfo")).unwrap_or_default();
    assert!(registro.starts_with("[Trash Info]\nPath=/"));
    assert!(registro.contains("/nota.txt\nDeletionDate="));
    assert!(!dir.path().join("nota.txt").exists());
}

#[test]
fn item_que_nao_existe_nao_deixa_registro_na_lixeira() {
    let Some((dir, disk)) = ambiente() else {
        panic!("sem pasta temporária")
    };
    assert!(disk.trash(&dir.path().join("fantasma.txt")).is_err());
    assert!(
        !dir.path()
            .join("Trash/info/fantasma.txt.trashinfo")
            .exists()
    );
}
