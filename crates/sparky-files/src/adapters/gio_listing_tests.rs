use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::gio_listing::{list, serves};
use super::home_trash::HomeTrash;
use super::local_disk::LocalDisk;
use crate::domain::entry::Entry;
use crate::ports::Disk;

fn em_microssegundos(moment: Option<SystemTime>) -> Option<u128> {
    moment
        .and_then(|moment| moment.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_micros())
}

fn resumo(mut entries: Vec<Entry>) -> Vec<(String, bool, u64, Option<u128>)> {
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
        .into_iter()
        .map(|entry| {
            let pasta = entry.is_directory();
            (
                entry.name,
                pasta,
                entry.size,
                em_microssegundos(entry.modified),
            )
        })
        .collect()
}

#[test]
fn listagem_pelo_gio_traz_o_mesmo_que_a_do_disco_numa_so_passagem() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("musica.mp3"), "12345").unwrap();
    fs::write(dir.path().join(".oculto"), "x").unwrap();
    fs::create_dir(dir.path().join("Fotos")).unwrap();
    let antigo = SystemTime::now() - Duration::from_secs(3600);
    fs::File::options()
        .write(true)
        .open(dir.path().join("musica.mp3"))
        .unwrap()
        .set_modified(antigo)
        .unwrap();
    let disco = LocalDisk::new(HomeTrash::new(dir.path().join("Trash")));
    let esperado = resumo(disco.list(dir.path()).unwrap());
    assert_eq!(resumo(list(dir.path()).unwrap()), esperado);
}

#[test]
fn pasta_inexistente_no_gio_vira_erro_de_nao_encontrado() {
    let erro = list(Path::new("/nao/existe/sparky-gio")).unwrap_err();
    assert_eq!(erro.kind(), io::ErrorKind::NotFound);
}

#[test]
fn so_pastas_dentro_da_raiz_do_gvfs_usam_o_gio() {
    let raiz = Path::new("/run/user/1000/gvfs");
    assert!(serves(raiz, &raiz.join("mtp:host=Pixel/Interno")));
    assert!(!serves(raiz, raiz));
    assert!(!serves(raiz, Path::new("/home/ana")));
    assert!(!serves(raiz, Path::new("/run/user/1000/gvfs-outro")));
}
