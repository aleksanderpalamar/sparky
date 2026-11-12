use std::fs;
use std::io;

use super::home_trash::HomeTrash;
use super::transfer::LocalTransfer;
use crate::domain::transfer::{Action, Step};
use crate::ports::Transfer;

fn passo(
    origem: std::path::PathBuf,
    destino: std::path::PathBuf,
    action: Action,
    replace: bool,
) -> Step {
    Step {
        source: origem,
        target: destino,
        action,
        replace,
    }
}

#[test]
fn copia_cancelada_nao_deixa_destino_nem_sobras() {
    let dir = tempfile::tempdir().unwrap();
    let origem = dir.path().join("video.mkv");
    fs::write(&origem, vec![7u8; 3 * 1024 * 1024]).unwrap();
    let transfer = LocalTransfer::new(HomeTrash::new(dir.path().join("Trash")), None);
    let destino = dir.path().join("copia.mkv");
    let erro = transfer
        .run(
            &passo(origem, destino.clone(), Action::Copy, false),
            &mut |_| false,
        )
        .unwrap_err();
    assert_eq!(erro.kind(), io::ErrorKind::Interrupted);
    assert!(!destino.exists());
    let nomes: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|i| i.file_name())
        .collect();
    assert_eq!(nomes, ["video.mkv"]);
}

#[test]
fn copia_informa_os_bytes_copiados() {
    let dir = tempfile::tempdir().unwrap();
    let origem = dir.path().join("dados.bin");
    fs::write(&origem, vec![1u8; 5000]).unwrap();
    let transfer = LocalTransfer::new(HomeTrash::new(dir.path().join("Trash")), None);
    let mut total = 0;
    transfer
        .run(
            &passo(origem, dir.path().join("b.bin"), Action::Copy, false),
            &mut |bytes| {
                total += bytes;
                true
            },
        )
        .unwrap();
    assert_eq!(total, 5000);
    assert_eq!(transfer.size(&dir.path().join("b.bin")), 5000);
}

#[test]
fn substituir_manda_o_item_antigo_para_a_lixeira() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("Docs")).unwrap();
    fs::write(dir.path().join("nota.txt"), "nova").unwrap();
    fs::write(dir.path().join("Docs/nota.txt"), "antiga").unwrap();
    let lixeira = dir.path().join("Trash");
    let transfer = LocalTransfer::new(HomeTrash::new(lixeira.clone()), None);
    let step = passo(
        dir.path().join("nota.txt"),
        dir.path().join("Docs/nota.txt"),
        Action::Move,
        true,
    );
    transfer.run(&step, &mut |_| true).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("Docs/nota.txt")).unwrap(),
        "nova"
    );
    assert_eq!(
        fs::read_to_string(lixeira.join("files/nota.txt")).unwrap(),
        "antiga"
    );
    assert!(!dir.path().join("nota.txt").exists());
}

#[test]
fn copia_direta_no_gvfs_nao_apaga_item_que_ja_existia() {
    let dir = tempfile::tempdir().unwrap();
    let remoto = dir.path().join("gvfs");
    fs::create_dir_all(remoto.join("telefone")).unwrap();
    fs::write(dir.path().join("foto.jpg"), "nova").unwrap();
    fs::write(remoto.join("telefone/foto.jpg"), "do telefone").unwrap();
    let transfer = LocalTransfer::new(
        HomeTrash::new(dir.path().join("Trash")),
        Some(remoto.clone()),
    );
    let step = passo(
        dir.path().join("foto.jpg"),
        remoto.join("telefone/foto.jpg"),
        Action::Copy,
        false,
    );
    let erro = transfer.run(&step, &mut |_| true).unwrap_err();
    assert_eq!(erro.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        fs::read_to_string(remoto.join("telefone/foto.jpg")).unwrap(),
        "do telefone"
    );
}

#[test]
fn copia_direta_no_gvfs_cancelada_remove_o_que_criou() {
    let dir = tempfile::tempdir().unwrap();
    let remoto = dir.path().join("gvfs");
    fs::create_dir_all(remoto.join("telefone")).unwrap();
    fs::write(dir.path().join("video.mp4"), vec![3u8; 4096]).unwrap();
    let transfer = LocalTransfer::new(
        HomeTrash::new(dir.path().join("Trash")),
        Some(remoto.clone()),
    );
    let step = passo(
        dir.path().join("video.mp4"),
        remoto.join("telefone/video.mp4"),
        Action::Copy,
        false,
    );
    assert!(transfer.run(&step, &mut |_| false).is_err());
    assert!(!remoto.join("telefone/video.mp4").exists());
}
