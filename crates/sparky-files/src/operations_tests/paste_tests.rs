use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::{FakeDisk, copia, recorte};
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::adapters::transfer::LocalTransfer;
use crate::domain::transfer::{Action, Conflict, Ending, Step};
use crate::operations::{Failure, conflicts, plan};
use crate::transfer_job::execute;

fn passos(planned: &[Step]) -> Vec<(PathBuf, PathBuf, Action, bool)> {
    planned
        .iter()
        .map(|step| {
            (
                step.source.clone(),
                step.target.clone(),
                step.action,
                step.replace,
            )
        })
        .collect()
}

fn caminho(texto: &str) -> PathBuf {
    PathBuf::from(texto)
}

#[test]
fn copiar_para_a_mesma_pasta_ganha_nome_de_copia_sem_perguntar() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let clip = copia(&["/h/foto.png"]);
    assert!(conflicts(&clip, Path::new("/h"), &disk).is_empty());
    let planned = plan(&clip, Path::new("/h"), &disk, Conflict::KeepBoth).unwrap();
    assert_eq!(
        passos(&planned.steps),
        [(
            caminho("/h/foto.png"),
            caminho("/h/foto (cópia).png"),
            Action::Copy,
            false
        )]
    );
}

#[test]
fn recortar_para_a_mesma_pasta_nao_planeja_nada() {
    let disk = FakeDisk::with(&["/h/foto.png"]);
    let planned = plan(
        &recorte(&["/h/foto.png"]),
        Path::new("/h"),
        &disk,
        Conflict::KeepBoth,
    )
    .unwrap();
    assert!(planned.steps.is_empty());
}

#[test]
fn recortar_para_outra_pasta_move_cada_item() {
    let disk = FakeDisk::with(&["/h/a.txt", "/h/b.txt"]);
    let clip = recorte(&["/h/a.txt", "/h/b.txt"]);
    let planned = plan(&clip, Path::new("/h/Docs"), &disk, Conflict::KeepBoth).unwrap();
    assert_eq!(
        passos(&planned.steps),
        [
            (
                caminho("/h/a.txt"),
                caminho("/h/Docs/a.txt"),
                Action::Move,
                false
            ),
            (
                caminho("/h/b.txt"),
                caminho("/h/Docs/b.txt"),
                Action::Move,
                false
            ),
        ]
    );
}

#[test]
fn nome_ocupado_no_destino_vira_conflito_e_segue_a_escolha() {
    let disk = FakeDisk::with(&["/h/a.txt", "/h/b.txt", "/h/Docs/a.txt"]);
    let clip = copia(&["/h/a.txt", "/h/b.txt"]);
    let into = Path::new("/h/Docs");
    assert_eq!(conflicts(&clip, into, &disk), ["a.txt"]);
    let ambos = plan(&clip, into, &disk, Conflict::KeepBoth).unwrap();
    assert_eq!(ambos.steps[0].target, caminho("/h/Docs/a (cópia).txt"));
    let substitui = plan(&clip, into, &disk, Conflict::Replace).unwrap();
    assert_eq!(
        passos(&substitui.steps)[0],
        (
            caminho("/h/a.txt"),
            caminho("/h/Docs/a.txt"),
            Action::Copy,
            true
        )
    );
    let pula = plan(&clip, into, &disk, Conflict::Skip).unwrap();
    assert_eq!(pula.skipped, 1);
    assert_eq!(passos(&pula.steps)[0].1, caminho("/h/Docs/b.txt"));
}

#[test]
fn colar_uma_pasta_dentro_dela_mesma_e_recusado() {
    let disk = FakeDisk::with(&["/h/Fotos"]);
    let planned = plan(
        &copia(&["/h/Fotos"]),
        Path::new("/h/Fotos/2026"),
        &disk,
        Conflict::KeepBoth,
    );
    assert!(matches!(planned, Err(Failure::IntoItself(name)) if name == "Fotos"));
}

#[test]
fn caminho_com_ponto_ponto_nao_permite_copiar_pasta_para_si_mesma() {
    let disk = FakeDisk::with(&["/h/Fotos"]);
    let planned = plan(
        &copia(&["/h/Fotos"]),
        Path::new("/h/../h/Fotos"),
        &disk,
        Conflict::KeepBoth,
    );
    assert!(matches!(planned, Err(Failure::IntoItself(name)) if name == "Fotos"));
}

#[test]
fn link_para_origem_nao_permite_copia_recursiva() {
    let dir = tempfile::tempdir().unwrap();
    let photos = dir.path().join("Fotos");
    let alias = dir.path().join("alias");
    fs::create_dir(&photos).unwrap();
    symlink(&photos, &alias).unwrap();
    let disk = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    let clip = crate::domain::clip::Clip::Copy(vec![photos.clone()]);
    let planned = plan(&clip, &alias, &disk, Conflict::KeepBoth);
    assert!(matches!(planned, Err(Failure::IntoItself(name)) if name == "Fotos"));
    assert!(fs::read_dir(&photos).unwrap().next().is_none());
}

#[test]
fn copia_link_quebrado_sem_resolver_o_alvo() {
    let dir = tempfile::tempdir().unwrap();
    let link = dir.path().join("atalho");
    symlink("ausente", &link).unwrap();
    let trash = HomeTrash::new(dir.path().join("Trash"));
    let disk = LocalDisk::new(trash.clone());
    let clip = crate::domain::clip::Clip::Copy(vec![link]);
    let planned = plan(&clip, dir.path(), &disk, Conflict::KeepBoth).unwrap();
    let transfer = LocalTransfer::new(trash, None);
    let outcome = execute(
        planned.steps,
        0,
        &transfer,
        &AtomicBool::new(false),
        &mut |_| {},
    );
    assert_eq!(outcome.ending, Ending::Finished);
    assert_eq!(
        fs::read_link(dir.path().join("atalho (cópia)")).ok(),
        Some("ausente".into())
    );
}
