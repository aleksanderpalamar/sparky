use std::{fs, path::PathBuf};
use toml::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn metadata() -> Value {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn backend_de_video_e_pipewire_sao_dependencias_de_execucao() {
    let manifest = metadata();
    let deb = manifest["package"]["metadata"]["deb"]["depends"]
        .as_str()
        .unwrap();
    let rpm = &manifest["package"]["metadata"]["generate-rpm"]["requires"];
    let arch = fs::read_to_string(root().join("packaging/arch/PKGBUILD")).unwrap();
    let depends = arch
        .split("depends=(")
        .nth(1)
        .unwrap()
        .split(')')
        .next()
        .unwrap();
    for name in [
        "xdg-desktop-portal",
        "xdg-desktop-portal-wlr",
        "xdg-desktop-portal-gtk",
        "pipewire",
    ] {
        assert!(
            deb.split(',')
                .any(|part| part.split_whitespace().next() == Some(name)),
            "DEB: {name}"
        );
        assert!(rpm.get(name).is_some(), "RPM: {name}");
        assert!(
            depends
                .lines()
                .any(|line| line.trim().trim_matches('\'').split(">=").next() == Some(name)),
            "Arch: {name}"
        );
    }
}

#[test]
fn compartilhamento_sempre_pede_escolha_e_limita_o_fps() {
    let config = fs::read_to_string(root().join("data/portals/sparky-wlr.conf")).unwrap();
    assert!(config.contains("[screencast]"));
    assert!(config.contains("chooser_type=dmenu"));
    assert!(config.contains("chooser_cmd=sparky-shot --share-source"));
    assert!(config.contains("max_fps=60"));
    assert!(!config.contains("output_name="));
}

#[test]
fn config_do_backend_e_instalada_no_diretorio_do_desktop() {
    let manifest = metadata();
    let deb = &manifest["package"]["metadata"]["deb"]["assets"];
    assert!(
        deb.as_array()
            .unwrap()
            .iter()
            .any(|asset| asset[1].as_str() == Some("etc/xdg/xdg-desktop-portal-wlr/sparky"))
    );
    let rpm = &manifest["package"]["metadata"]["generate-rpm"]["assets"];
    let config = rpm
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["dest"].as_str() == Some("/etc/xdg/xdg-desktop-portal-wlr/sparky"))
        .unwrap();
    assert_eq!(config["config"].as_str(), Some("noreplace"));
    let arch = fs::read_to_string(root().join("packaging/arch/PKGBUILD")).unwrap();
    assert!(arch.contains("backup=('etc/xdg/xdg-desktop-portal-wlr/sparky')"));
    assert!(
        arch.split("package_sparky-wallpapers()")
            .nth(1)
            .unwrap()
            .contains("backup=()")
    );
    assert!(arch.contains(
        "data/portals/sparky-wlr.conf \"$pkgdir/etc/xdg/xdg-desktop-portal-wlr/sparky\""
    ));
}
