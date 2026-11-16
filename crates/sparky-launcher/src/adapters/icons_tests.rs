use std::fs;
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};

use super::icons::Icons;
use crate::domain::icon::{Icon, Picture};

fn tema(raiz: &Path) -> PathBuf {
    let tema = raiz.join("icons/Teste");
    for pasta in ["32x32/apps", "scalable/actions"] {
        assert!(fs::create_dir_all(tema.join(pasta)).is_ok());
    }
    let indice = "[Icon Theme]\nDirectories=32x32/apps,scalable/actions\n\n[32x32/apps]\nSize=32\n\n[scalable/actions]\nSize=16\nType=Scalable\n";
    assert!(fs::write(tema.join("index.theme"), indice).is_ok());
    let firefox = RgbaImage::from_pixel(64, 64, Rgba([10, 20, 30, 255]));
    assert!(firefox.save(tema.join("32x32/apps/firefox.png")).is_ok());
    assert!(fs::write(tema.join("scalable/actions/kvantum.svg"), "<svg/>").is_ok());
    assert!(
        fs::write(
            tema.join("scalable/actions/system-reboot-symbolic.svg"),
            "<svg/>"
        )
        .is_ok()
    );
    assert!(fs::create_dir_all(raiz.join("pixmaps")).is_ok());
    assert!(fs::write(raiz.join("pixmaps/antigo.png"), "png").is_ok());
    raiz.join("icons")
}

fn icones(raiz: &Path) -> Icons {
    Icons::new(vec![tema(raiz)], "Teste")
}

#[test]
fn o_icone_do_aplicativo_vem_do_tema_dos_pixmaps_ou_do_caminho_absoluto() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let icones = icones(dir.path());
    let nomeado = |nome: &str| icones.path(&Icon::Named(nome.to_owned()));
    assert_eq!(
        nomeado("firefox"),
        Some(dir.path().join("icons/Teste/32x32/apps/firefox.png"))
    );
    assert_eq!(
        nomeado("antigo"),
        Some(dir.path().join("pixmaps/antigo.png"))
    );
    let absoluto = dir.path().join("pixmaps/antigo.png");
    assert_eq!(nomeado(&absoluto.to_string_lossy()), Some(absoluto));
    assert_eq!(
        nomeado("antigo.png"),
        Some(dir.path().join("pixmaps/antigo.png"))
    );
    assert_eq!(nomeado("inexistente"), None);
    assert_eq!(nomeado("../antigo"), None);
}

#[test]
fn o_icone_simbolico_usa_o_primeiro_nome_que_o_tema_tiver() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let icones = icones(dir.path());
    let reiniciar = Icon::Symbolic(&["system-restart-symbolic", "system-reboot-symbolic"]);
    assert_eq!(
        icones.path(&reiniciar),
        Some(
            dir.path()
                .join("icons/Teste/scalable/actions/system-reboot-symbolic.svg")
        )
    );
    assert_eq!(icones.path(&Icon::Symbolic(&["nada-symbolic"])), None);
}

#[test]
fn o_mapa_liga_cada_aplicativo_ao_seu_icone_e_pula_os_sem_icone() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let icones = icones(dir.path());
    let app = |id: &str, icon: Option<Icon>| crate::domain::app::App {
        id: id.to_owned(),
        name: id.to_owned(),
        detail: None,
        keywords: Vec::new(),
        command: id.to_owned(),
        runs: crate::domain::app::Runs::Directly,
        icon,
        categories: Vec::new(),
    };
    let mapa = icones.resolve(&[
        app("firefox", Some(Icon::Named("firefox".to_owned()))),
        app("kitty", None),
        app("kvantum", Some(Icon::Named("kvantum".to_owned()))),
        app("sumido", Some(Icon::Named("sumido".to_owned()))),
        app("quebrado", Some(Icon::Named("antigo".to_owned()))),
    ]);
    assert_eq!(mapa.len(), 2);
    let Some(Picture::Raster(firefox)) = mapa.get("firefox") else {
        panic!("firefox vira bitmap");
    };
    assert_eq!((firefox.width, firefox.height), (48, 48));
    assert_eq!(
        mapa.get("kvantum"),
        Some(&Picture::Vector(
            dir.path().join("icons/Teste/scalable/actions/kvantum.svg")
        ))
    );
}

#[test]
fn icones_soltos_da_barra_tambem_sao_achados_pela_chave() {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("pasta temporária");
    };
    let mapa = icones(dir.path()).named(&[
        (
            "@reboot".to_owned(),
            Icon::Symbolic(&["system-reboot-symbolic"]),
        ),
        ("@nada".to_owned(), Icon::Symbolic(&["nao-existe-symbolic"])),
    ]);
    assert_eq!(mapa.len(), 1);
    assert!(matches!(mapa.get("@reboot"), Some(Picture::Vector(_))));
}
