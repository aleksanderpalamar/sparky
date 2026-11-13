use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use super::disk_trash::{mount_points, mount_root, prepare};
use super::home_trash::HomeTrash;
use crate::ports::TrashStore;

const UID: u32 = 1000;

#[test]
fn raiz_do_disco_e_o_ultimo_ancestral_no_mesmo_dispositivo() {
    let dispositivo = |caminho: &Path| match caminho.starts_with("/run/media/ana/USB") {
        true => Some(2),
        false => Some(1),
    };
    assert_eq!(
        mount_root(Path::new("/run/media/ana/USB/Fotos/praia.png"), dispositivo),
        PathBuf::from("/run/media/ana/USB")
    );
}

#[test]
fn sem_lixeira_compartilhada_cria_a_pasta_do_usuario_privada() {
    let disco = tempfile::tempdir().unwrap();
    let raiz = prepare(disco.path(), UID).unwrap();
    assert_eq!(raiz, disco.path().join(".Trash-1000"));
    assert_eq!(
        fs::metadata(&raiz).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[test]
fn lixeira_compartilhada_com_sticky_bit_e_usada() {
    let disco = tempfile::tempdir().unwrap();
    let compartilhada = disco.path().join(".Trash");
    fs::create_dir(&compartilhada).unwrap();
    fs::set_permissions(&compartilhada, fs::Permissions::from_mode(0o1777)).unwrap();
    assert_eq!(
        prepare(disco.path(), rustix::process::getuid().as_raw()).unwrap(),
        compartilhada.join(rustix::process::getuid().as_raw().to_string())
    );
}

#[test]
fn lixeira_compartilhada_sem_sticky_bit_e_ignorada() {
    let disco = tempfile::tempdir().unwrap();
    fs::create_dir(disco.path().join(".Trash")).unwrap();
    let uid = rustix::process::getuid().as_raw();
    assert_eq!(
        prepare(disco.path(), uid).unwrap(),
        disco.path().join(format!(".Trash-{uid}"))
    );
}

#[test]
fn lixeira_do_usuario_que_e_link_e_recusada() {
    let disco = tempfile::tempdir().unwrap();
    let uid = rustix::process::getuid().as_raw();
    fs::create_dir(disco.path().join("outra")).unwrap();
    symlink(
        disco.path().join("outra"),
        disco.path().join(format!(".Trash-{uid}")),
    )
    .unwrap();
    assert!(prepare(disco.path(), uid).is_err());
}

#[test]
fn pontos_de_montagem_ignoram_sistemas_virtuais_e_de_rede() {
    let mounts = "proc /proc proc rw 0 0\n/dev/sdb1 /run/media/ana/MEU\\040PENDRIVE vfat rw 0 0\nserver:/x /mnt/nfs nfs4 rw 0 0\n/dev/nvme0n1p2 / btrfs rw 0 0\n";
    assert_eq!(
        mount_points(mounts),
        [
            PathBuf::from("/run/media/ana/MEU PENDRIVE"),
            PathBuf::from("/")
        ]
    );
}

#[test]
fn item_de_outro_disco_volta_pelo_caminho_relativo_a_raiz() {
    let dir = tempfile::tempdir().unwrap();
    let disco = dir.path().join("USB");
    fs::create_dir_all(disco.join("Fotos")).unwrap();
    fs::write(disco.join("Fotos/praia.png"), "png").unwrap();
    let raiz = prepare(&disco, rustix::process::getuid().as_raw()).unwrap();
    let lixeira = HomeTrash::on_disk(raiz.clone(), disco.clone());
    let entrada = lixeira.put(&disco.join("Fotos/praia.png")).unwrap();
    let registro = fs::read_to_string(raiz.join("info/praia.png.trashinfo")).unwrap();
    assert!(registro.contains("\nPath=Fotos/praia.png\n"));
    assert_eq!(lixeira.label().as_deref(), Some("USB"));
    assert_eq!(
        lixeira.restore(&entrada.name).unwrap(),
        disco.join("Fotos/praia.png")
    );
    assert!(disco.join("Fotos/praia.png").exists());
}
