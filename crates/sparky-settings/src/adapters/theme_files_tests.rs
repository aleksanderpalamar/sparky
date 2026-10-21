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

#[test]
fn os_icones_do_tema_vem_do_icons_theme() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let system = dir.path().join("sistema");
    fs::create_dir_all(system.join("azul"))?;
    fs::write(system.join("azul/colors.toml"), "accent = \"#010203\"")?;
    fs::write(system.join("azul/icons.theme"), "Yaru-blue\n")?;
    let store = ThemeFiles::new(vec![system], dir.path().join("cfg/colors.toml"), dir.path().join("padrao/colors.toml"));
    let catalog = store.load()?;
    assert_eq!(catalog.themes[0].icons(), Some("Yaru-blue"));
    assert_eq!(catalog.themes[1].icons(), None);
    Ok(())
}

#[test]
fn aplicar_grava_os_icones_no_gtk_3_e_4_guardando_o_resto() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let gtk3 = dir.path().join("gtk-3.0");
    let gtk4 = dir.path().join("gtk-4.0");
    fs::create_dir_all(&gtk3)?;
    fs::write(gtk3.join("settings.ini"), "[Settings]\ngtk-theme-name=Adwaita-dark\n")?;
    let store = ThemeFiles::new(Vec::new(), dir.path().join("cfg/colors.toml"), dir.path().join("padrao/colors.toml")).with_gtk(vec![gtk3.clone(), gtk4.clone()]);
    let theme = Theme::parse("azul", "accent = \"#010203\"\n").map_err(std::io::Error::other)?.with_icons(Some("Yaru-blue".to_owned()));
    store.apply(&theme)?;
    assert_eq!(fs::read_to_string(gtk3.join("settings.ini"))?, "[Settings]\ngtk-icon-theme-name=Yaru-blue\ngtk-theme-name=Adwaita-dark\n");
    assert_eq!(fs::read_to_string(gtk4.join("settings.ini"))?, "[Settings]\ngtk-icon-theme-name=Yaru-blue\n");
    Ok(())
}

#[test]
fn tema_sem_icones_nao_mexe_no_gtk() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let gtk3 = dir.path().join("gtk-3.0");
    let store = ThemeFiles::new(Vec::new(), dir.path().join("cfg/colors.toml"), dir.path().join("padrao/colors.toml")).with_gtk(vec![gtk3.clone()]);
    store.apply(&Theme::builtin())?;
    assert!(!gtk3.exists());
    Ok(())
}

#[test]
fn os_temas_do_pacote_sao_validos_e_trazem_icones() -> std::io::Result<()> {
    let pacote = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/themes");
    let pastas = fs::read_dir(&pacote)?.filter_map(Result::ok).filter(|entrada| entrada.path().is_dir()).count();
    let dir = tempfile::tempdir()?;
    let store = ThemeFiles::new(vec![pacote], dir.path().join("cfg/colors.toml"), dir.path().join("padrao/colors.toml"));
    let catalog = store.load()?;
    assert_eq!(catalog.themes.len(), pastas);
    assert!(pastas >= 20);
    for tema in &catalog.themes {
        assert!(tema.icons().is_some_and(|nome| nome.starts_with("Yaru")), "{} sem ícones", tema.name);
    }
    Ok(())
}
