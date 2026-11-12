use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use tempfile::TempDir;

use crate::adapters::finder::TreeFinder;
use crate::domain::entry::Kind;
use crate::domain::listing::Hidden;
use crate::domain::search::{Finish, Query};
use crate::ports::Finder;

fn arvore() -> TempDir {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("sem pasta temporária")
    };
    let raiz = dir.path();
    for pasta in ["Fotos/Praia", ".oculta", "Docs"] {
        assert!(fs::create_dir_all(raiz.join(pasta)).is_ok());
    }
    for arquivo in [
        "Fotos/Praia/Fóto 1.jpg",
        "Fotos/nota.txt",
        ".oculta/foto.png",
        "Docs/foto.pdf",
    ] {
        assert!(fs::write(raiz.join(arquivo), "x").is_ok());
    }
    assert!(symlink(raiz, raiz.join("Docs/volta")).is_ok());
    dir
}

fn buscar(
    raiz: &Path,
    texto: &str,
    hidden: Hidden,
    limite: usize,
) -> (Vec<(String, Kind)>, Finish) {
    let Some(query) = Query::new(texto) else {
        panic!("busca inválida")
    };
    let mut achados = Vec::new();
    let fim =
        TreeFinder::new(limite).find(raiz, &query, hidden, &AtomicBool::new(false), &mut |lote| {
            achados.extend(lote.into_iter().map(|entry| (entry.name, entry.kind)));
        });
    achados.sort_by(|a, b| a.0.cmp(&b.0));
    (achados, fim)
}

#[test]
fn acha_em_subpastas_sem_ocultos_e_sem_seguir_links() {
    let dir = arvore();
    let (achados, fim) = buscar(dir.path(), "foto", Hidden::Concealed, 100);
    assert_eq!(
        achados,
        [
            ("Docs/foto.pdf".to_owned(), Kind::File),
            ("Fotos".to_owned(), Kind::Directory),
            ("Fotos/Praia/Fóto 1.jpg".to_owned(), Kind::File),
        ]
    );
    assert_eq!(fim, Finish::Complete);
}

#[test]
fn com_ocultos_visiveis_entra_nas_pastas_ocultas() {
    let dir = arvore();
    let (achados, _) = buscar(dir.path(), "foto.png", Hidden::Shown, 100);
    assert_eq!(achados, [(".oculta/foto.png".to_owned(), Kind::File)]);
}

#[test]
fn para_no_limite_e_avisa() {
    let dir = arvore();
    let (achados, fim) = buscar(dir.path(), "o", Hidden::Concealed, 2);
    assert_eq!(achados.len(), 2);
    assert_eq!(fim, Finish::Limited);
}

#[test]
fn cancelada_antes_de_comecar_nao_acha_nada() {
    let dir = arvore();
    let Some(query) = Query::new("foto") else {
        return;
    };
    let mut achados = 0;
    let fim = TreeFinder::new(100).find(
        dir.path(),
        &query,
        Hidden::Concealed,
        &AtomicBool::new(true),
        &mut |lote| {
            achados += lote.len();
        },
    );
    assert_eq!((achados, fim), (0, Finish::Cancelled));
}
