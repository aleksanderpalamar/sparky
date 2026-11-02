use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use tempfile::TempDir;

use crate::adapters::appearance::{Snapshot, read_limited, store};

fn pasta() -> TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn le_o_arquivo_comum_inteiro() {
    let dir = pasta();
    let arquivo = dir.path().join("fundo.png");
    assert!(fs::write(&arquivo, b"imagem").is_ok());
    assert_eq!(read_limited(&arquivo, 100).ok(), Some(b"imagem".to_vec()));
}

#[test]
fn arquivo_grande_demais_pasta_ou_ausente_nao_e_lido() {
    let dir = pasta();
    let grande = dir.path().join("grande");
    assert!(fs::write(&grande, vec![0u8; 101]).is_ok());
    assert!(read_limited(&grande, 100).is_err());
    assert!(read_limited(dir.path(), 100).is_err());
    assert!(read_limited(&dir.path().join("nao-existe"), 100).is_err());
}

#[test]
fn fifo_e_recusado_sem_travar() {
    let dir = pasta();
    let fifo = dir.path().join("fifo");
    assert!(mkfifo(&fifo, Mode::S_IRUSR | Mode::S_IWUSR).is_ok());
    let comeco = Instant::now();
    assert!(read_limited(&fifo, 100).is_err());
    assert!(comeco.elapsed() < Duration::from_secs(1));
}

#[test]
fn guarda_as_cores_e_a_imagem_so_para_o_dono_e_o_grupo() {
    let dir = pasta();
    let snapshot = Snapshot {
        colors: Some(b"accent = \"#010203\"".to_vec()),
        wallpaper: Some(b"imagem".to_vec()),
    };
    assert!(store(dir.path(), &snapshot, None).is_ok());
    assert_eq!(
        fs::read(dir.path().join("colors.toml")).ok(),
        Some(b"accent = \"#010203\"".to_vec())
    );
    assert_eq!(
        fs::read(dir.path().join("wallpaper")).ok(),
        Some(b"imagem".to_vec())
    );
    let modo = fs::metadata(dir.path().join("wallpaper"))
        .map(|meta| meta.permissions().mode() & 0o777)
        .ok();
    assert_eq!(modo, Some(0o640));
}

#[test]
fn sem_escolha_a_copia_antiga_some() {
    let dir = pasta();
    let antes = Snapshot {
        colors: Some(b"x".to_vec()),
        wallpaper: Some(b"y".to_vec()),
    };
    assert!(store(dir.path(), &antes, None).is_ok());
    let depois = Snapshot {
        colors: None,
        wallpaper: None,
    };
    assert!(store(dir.path(), &depois, None).is_ok());
    assert!(!dir.path().join("colors.toml").exists());
    assert!(!dir.path().join("wallpaper").exists());
}
