use std::fs;
use std::path::Path;

use super::Icons;
use crate::domain::tray::TrayIcon;

fn theme(root: &Path) -> Icons {
    fs::create_dir_all(root.join("Yaru-magenta")).unwrap();
    fs::create_dir_all(root.join("Yaru/scalable/devices")).unwrap();
    fs::write(
        root.join("Yaru-magenta/index.theme"),
        "[Icon Theme]\nInherits=Yaru\n",
    )
    .unwrap();
    fs::write(
        root.join("Yaru/index.theme"),
        "[Icon Theme]\nDirectories=scalable/devices\n",
    )
    .unwrap();
    Icons::new(vec![root.to_owned()], "Yaru-magenta")
}

#[test]
fn procura_simbolico_no_tema_herdado_do_yaru() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root
        .path()
        .join("Yaru/scalable/devices/network-wired-symbolic.svg");
    fs::write(&expected, "<svg/>").unwrap();
    assert_eq!(
        icons.resolve("nm-device-wired", None),
        Some(TrayIcon::Symbolic(expected))
    );
}

#[test]
fn simbolico_tem_prioridade_sobre_pixmap_colorido_do_aplicativo() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root
        .path()
        .join("Yaru/scalable/devices/network-wired-symbolic.svg");
    fs::write(&expected, "<svg/>").unwrap();
    fs::write(root.path().join("nm-device-wired.png"), "png").unwrap();
    assert_eq!(
        icons.resolve("nm-device-wired", root.path().to_str()),
        Some(TrayIcon::Symbolic(expected))
    );
}

#[test]
fn preserva_icones_proprios_e_nomes_com_ponto_sem_inventar_arquivos() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root.path().join("com.example.App.png");
    fs::write(&expected, "png").unwrap();
    assert_eq!(
        icons.resolve("com.example.App", root.path().to_str()),
        Some(TrayIcon::File(expected.clone()))
    );
    assert_eq!(
        icons.resolve(expected.to_str().unwrap(), None),
        Some(TrayIcon::File(expected))
    );
    assert_eq!(icons.resolve("inexistente", None), None);
    assert_eq!(icons.resolve("../inexistente", root.path().to_str()), None);
}

#[test]
fn reconhece_icone_simbolico_fornecido_por_caminho_absoluto() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root.path().join("custom-symbolic.svg");
    fs::write(&expected, "<svg/>").unwrap();
    assert_eq!(
        icons.resolve(expected.to_str().unwrap(), None),
        Some(TrayIcon::Symbolic(expected))
    );
}

#[test]
fn preserva_o_fallback_de_pixmaps_do_sistema() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("pixmaps");
    fs::create_dir_all(&directory).unwrap();
    let expected = directory.join("custom-app.png");
    fs::write(&expected, "png").unwrap();
    let icons = Icons::new(vec![root.path().join("icons")], "hicolor");
    assert_eq!(
        icons.resolve("custom-app", None),
        Some(TrayIcon::File(expected))
    );
}

#[test]
fn bitmap_com_nome_simbolico_continua_sendo_renderizado_como_imagem() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root.path().join("bitmap-symbolic.png");
    fs::write(&expected, "png").unwrap();
    assert_eq!(
        icons.resolve(expected.to_str().unwrap(), None),
        Some(TrayIcon::File(expected))
    );
}

#[test]
fn icone_mono_de_tray_e_sufixo_de_extensao_sao_preservados() {
    let root = tempfile::tempdir().unwrap();
    let icons = theme(root.path());
    let expected = root
        .path()
        .join("Yaru/scalable/devices/steam_tray_mono.svg");
    fs::write(&expected, "<svg/>").unwrap();
    assert_eq!(
        icons.resolve("steam_tray_mono.svg", None),
        Some(TrayIcon::Symbolic(expected))
    );
}
