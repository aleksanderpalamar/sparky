use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use super::server::Error;

pub fn publish(directory: &Path, name: &OsStr) -> std::io::Result<()> {
    let pending = directory.join("ready.tmp");
    fs::write(&pending, name.as_encoded_bytes())?;
    fs::rename(pending, directory.join("ready"))
}

pub fn socket(directory: &Path) -> Option<PathBuf> {
    let name = fs::read_to_string(directory.join("ready")).ok()?;
    match name.is_empty() {
        true => None,
        false => Some(directory.join(name)),
    }
}

#[test]
fn ready_vazio_ainda_nao_esta_pronto() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    fs::write(directory.path().join("ready"), "")?;
    assert_eq!(socket(directory.path()), None);
    Ok(())
}

#[test]
fn ready_publicado_aponta_para_o_socket_sem_arquivo_parcial() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    assert_eq!(socket(directory.path()), None);
    publish(directory.path(), OsStr::new("wayland-7"))?;
    assert_eq!(
        socket(directory.path()),
        Some(directory.path().join("wayland-7"))
    );
    assert!(!directory.path().join("ready.tmp").exists());
    Ok(())
}
