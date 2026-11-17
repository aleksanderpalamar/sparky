use std::fs;
use std::path::PathBuf;

use sparky_theme::{Palette, Rgb};

use crate::adapters::theme_files::ThemeFiles;
use crate::ports::themes::ThemeStore;

#[test]
fn os_temas_sparky_trazem_icones_e_papeis_de_parede() -> std::io::Result<()> {
    let dados = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let dir = tempfile::tempdir()?;
    let store = ThemeFiles::new(
        vec![dados.join("themes")],
        dir.path().join("cfg/colors.toml"),
        dir.path().join("padrao/colors.toml"),
    )
    .with_wallpapers(vec![dados.join("wallpapers")]);
    let catalog = store.load()?;
    for (nome, icones) in [("sparky", "Yaru"), ("sparky-98", "Yaru-yellow")] {
        let Some(tema) = catalog.themes.iter().find(|tema| tema.name == nome) else {
            panic!("{nome} ausente");
        };
        assert_eq!(tema.icons(), Some(icones));
        assert!(!tema.wallpapers().is_empty(), "{nome} sem papel de parede");
    }
    Ok(())
}

#[test]
fn sem_diretorios_o_tema_padrao_sparky_continua_disponivel() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let store = ThemeFiles::new(
        vec![dir.path().join("ausente")],
        dir.path().join("cfg/colors.toml"),
        dir.path().join("padrao/colors.toml"),
    );
    let catalog = store.load()?;
    assert_eq!(catalog.themes.len(), 1);
    assert_eq!(catalog.themes[0].name, "sparky");
    assert_eq!(catalog.themes[0].palette(), Palette::SPARKY);
    assert_eq!(catalog.current, Palette::SPARKY);
    Ok(())
}

#[test]
fn catalogo_do_usuario_nao_muda_a_paleta_atual_antes_de_aplicar() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let user = dir.path().join("usuario");
    let system = dir.path().join("sistema");
    fs::create_dir_all(user.join("sparky"))?;
    fs::create_dir_all(system.join("sparky"))?;
    fs::write(user.join("sparky/colors.toml"), "accent = \"#010203\"")?;
    fs::write(system.join("sparky/colors.toml"), "accent = \"#abcdef\"")?;
    let store = ThemeFiles::new(
        vec![user, system.clone()],
        dir.path().join("cfg/colors.toml"),
        system.join("sparky/colors.toml"),
    );
    let catalog = store.load()?;
    assert_eq!(catalog.themes[0].palette().accent, Rgb::new(1, 2, 3));
    assert_eq!(catalog.current.accent, Rgb::new(0xab, 0xcd, 0xef));
    Ok(())
}
