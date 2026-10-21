use std::fs;

use sparky_theme::{Palette, Rgb};

use crate::adapters::theme_files::ThemeFiles;
use crate::domain::system::themes::Theme;
use crate::ports::themes::ThemeStore;

#[test]
fn catalogo_prioriza_o_usuario_e_ignora_arquivos_invalidos() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let user = dir.path().join("usuario");
    let system = dir.path().join("sistema");
    fs::create_dir_all(user.join("azul"))?;
    fs::create_dir_all(system.join("azul"))?;
    fs::create_dir_all(system.join("invalido"))?;
    fs::write(user.join("azul/colors.toml"), "accent = \"#010203\"")?;
    fs::write(system.join("azul/colors.toml"), "accent = \"#ff0000\"")?;
    fs::write(system.join("invalido/colors.toml"), "accent = \"azul\"")?;
    let store = ThemeFiles::new(vec![user, system.clone()], dir.path().join("cfg/colors.toml"), system.join("tokyo-night/colors.toml"));
    let catalog = store.load()?;
    assert_eq!(catalog.themes.len(), 2);
    assert_eq!(catalog.themes[0].name, "azul");
    assert_eq!(catalog.themes[0].palette().accent, Rgb::new(1, 2, 3));
    Ok(())
}

#[test]
fn aplicar_grava_as_cores_e_a_leitura_reconhece_a_nova_paleta() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let target = dir.path().join("cfg/colors.toml");
    let store = ThemeFiles::new(Vec::new(), target.clone(), dir.path().join("padrao/colors.toml"));
    let theme = Theme::parse("azul", "accent = \"#010203\"\n").map_err(std::io::Error::other)?;
    store.apply(&theme)?;
    assert_eq!(fs::read_to_string(target)?, theme.contents());
    assert_eq!(store.load()?.current.accent, Rgb::new(1, 2, 3));
    Ok(())
}

#[test]
fn erro_de_gravacao_e_devolvido_sem_criar_arquivo_em_outro_lugar() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let blocked = dir.path().join("arquivo");
    fs::write(&blocked, "preservado")?;
    let store = ThemeFiles::new(Vec::new(), blocked.join("colors.toml"), dir.path().join("padrao/colors.toml"));
    assert!(store.apply(&Theme::builtin()).is_err());
    assert_eq!(fs::read_to_string(blocked)?, "preservado");
    Ok(())
}

#[test]
fn sem_diretorios_o_tokyo_night_continua_disponivel() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let store = ThemeFiles::new(vec![dir.path().join("ausente")], dir.path().join("cfg/colors.toml"), dir.path().join("padrao/colors.toml"));
    let catalog = store.load()?;
    assert_eq!(catalog.themes.len(), 1);
    assert_eq!(catalog.themes[0].palette(), Palette::TOKYO_NIGHT);
    assert_eq!(catalog.current, Palette::TOKYO_NIGHT);
    Ok(())
}

#[test]
fn catalogo_do_usuario_nao_muda_a_paleta_atual_antes_de_aplicar() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let user = dir.path().join("usuario");
    let system = dir.path().join("sistema");
    fs::create_dir_all(user.join("tokyo-night"))?;
    fs::create_dir_all(system.join("tokyo-night"))?;
    fs::write(user.join("tokyo-night/colors.toml"), "accent = \"#010203\"")?;
    fs::write(system.join("tokyo-night/colors.toml"), "accent = \"#abcdef\"")?;
    let store = ThemeFiles::new(vec![user, system.clone()], dir.path().join("cfg/colors.toml"), system.join("tokyo-night/colors.toml"));
    let catalog = store.load()?;
    assert_eq!(catalog.themes[0].palette().accent, Rgb::new(1, 2, 3));
    assert_eq!(catalog.current.accent, Rgb::new(0xab, 0xcd, 0xef));
    Ok(())
}
