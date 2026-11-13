use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use super::thumbnail_cache::SharedThumbnails;
use super::thumbnail_png::{self, Stamp};
use crate::domain::thumbnail::Thumbnail;
use crate::domain::thumbnail_key::{cache_name, file_uri};
use crate::ports::ThumbnailSource;

struct Contada(Arc<AtomicUsize>);

impl ThumbnailSource for Contada {
    fn load(&self, _path: &Path) -> Option<Thumbnail> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Some(quadrado(2))
    }
}

struct Cena {
    _dir: tempfile::TempDir,
    root: PathBuf,
    foto: PathBuf,
    chamadas: Arc<AtomicUsize>,
    cache: SharedThumbnails<Contada>,
}

impl Cena {
    fn nova() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("cache/thumbnails");
        let foto = dir.path().join("praia 1.png");
        fs::write(&foto, b"pixels").unwrap();
        let chamadas = Arc::new(AtomicUsize::new(0));
        let cache = SharedThumbnails::new(root.clone(), Contada(chamadas.clone()));
        Self {
            _dir: dir,
            root,
            foto,
            chamadas,
            cache,
        }
    }

    fn chamadas(&self) -> usize {
        self.chamadas.load(Ordering::SeqCst)
    }

    fn selo(&self) -> Stamp {
        let metadata = fs::metadata(&self.foto).unwrap();
        let mtime = metadata.modified().unwrap();
        Stamp {
            uri: file_uri(&self.foto),
            mtime: mtime
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            size: metadata.len(),
        }
    }

    fn gravado(&self, tamanho: &str) -> PathBuf {
        self.root
            .join(tamanho)
            .join(cache_name(&file_uri(&self.foto)))
    }
}

fn quadrado(lado: u32) -> Thumbnail {
    Thumbnail {
        width: lado,
        height: lado,
        rgba: vec![200; (lado * lado * 4) as usize],
    }
}

#[test]
fn miniatura_nova_vai_para_o_cache_compartilhado_e_e_reaproveitada() {
    let cena = Cena::nova();
    assert_eq!(cena.cache.load(&cena.foto), Some(quadrado(2)));
    let gravado = cena.gravado("normal");
    assert_eq!(
        fs::metadata(&gravado).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(cena.root.join("normal"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        thumbnail_png::read(&gravado, &cena.selo()),
        Some(quadrado(2))
    );
    assert_eq!(cena.cache.load(&cena.foto), Some(quadrado(2)));
    assert_eq!(cena.chamadas(), 1);
}

#[test]
fn arquivo_modificado_gera_miniatura_nova() {
    let cena = Cena::nova();
    cena.cache.load(&cena.foto);
    let depois = SystemTime::now() + Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&cena.foto)
        .unwrap()
        .set_modified(depois)
        .unwrap();
    cena.cache.load(&cena.foto);
    assert_eq!(cena.chamadas(), 2);
}

#[test]
fn miniatura_grande_de_outro_aplicativo_e_lida_e_reduzida() {
    let cena = Cena::nova();
    let nome = cache_name(&file_uri(&cena.foto));
    thumbnail_png::write(
        &cena.root.join("large"),
        &nome,
        &cena.selo(),
        &quadrado(256),
    )
    .unwrap();
    let lida = cena.cache.load(&cena.foto).unwrap();
    assert_eq!((lida.width, lida.height), (128, 128));
    assert_eq!(cena.chamadas(), 0);
}

#[test]
fn miniatura_de_outro_arquivo_com_o_mesmo_nome_e_ignorada() {
    let cena = Cena::nova();
    let selo = Stamp {
        uri: "file:///outro.png".to_owned(),
        ..cena.selo()
    };
    let nome = cache_name(&file_uri(&cena.foto));
    thumbnail_png::write(&cena.root.join("normal"), &nome, &selo, &quadrado(4)).unwrap();
    assert_eq!(cena.cache.load(&cena.foto), Some(quadrado(2)));
    assert_eq!(cena.chamadas(), 1);
}

#[test]
fn arquivos_dentro_do_proprio_cache_nao_sao_gravados_nele() {
    let cena = Cena::nova();
    let interna = cena.root.join("normal/abc.png");
    fs::create_dir_all(interna.parent().unwrap()).unwrap();
    fs::write(&interna, b"pixels").unwrap();
    cena.cache.load(&interna);
    assert_eq!(fs::read_dir(cena.root.join("normal")).unwrap().count(), 1);
}

#[test]
fn miniatura_com_o_mesmo_segundo_e_outro_tamanho_e_refeita() {
    let cena = Cena::nova();
    let selo = Stamp {
        size: cena.selo().size + 1,
        ..cena.selo()
    };
    let nome = cache_name(&file_uri(&cena.foto));
    thumbnail_png::write(&cena.root.join("normal"), &nome, &selo, &quadrado(4)).unwrap();
    assert_eq!(cena.cache.load(&cena.foto), Some(quadrado(2)));
    assert_eq!(cena.chamadas(), 1);
}
