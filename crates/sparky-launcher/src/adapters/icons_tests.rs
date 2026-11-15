use std::fs;
use std::path::{Path, PathBuf};

use super::icons::Icons;
use crate::domain::icon::Icon;

fn tema(raiz: &Path) -> PathBuf {
    let tema = raiz.join("icons/Teste");
    for pasta in ["32x32/apps", "scalable/actions"] {
        assert!(fs::create_dir_all(tema.join(pasta)).is_ok());
    }
    let indice = "[Icon Theme]\nDirectories=32x32/apps,scalable/actions\n\n[32x32/apps]\nSize=32\n\n[scalable/actions]\nSize=16\nType=Scalable\n";
    assert!(fs::write(tema.join("index.theme"), indice).is_ok());
    assert!(fs::write(tema.join("32x32/apps/firefox.png"), "png").is_ok());
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
    };
    let mapa = icones.resolve(&[
        app("firefox", Some(Icon::Named("firefox".to_owned()))),
        app("kitty", None),
        app("sumido", Some(Icon::Named("sumido".to_owned()))),
    ]);
    assert_eq!(mapa.len(), 1);
    assert!(mapa.contains_key("firefox"));
}
