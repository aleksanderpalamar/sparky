use std::path::PathBuf;

use crate::adapters::theme_files::ThemeFiles;
use crate::ports::themes::ThemeStore;

#[test]
fn os_temas_spark_trazem_icones_e_papeis_de_parede() -> std::io::Result<()> {
    let dados = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let dir = tempfile::tempdir()?;
    let store = ThemeFiles::new(
        vec![dados.join("themes")],
        dir.path().join("cfg/colors.toml"),
        dir.path().join("padrao/colors.toml"),
    )
    .with_wallpapers(vec![dados.join("wallpapers")]);
    let catalog = store.load()?;
    for (nome, icones) in [("spark", "Yaru"), ("spark-98", "Yaru-yellow")] {
        let Some(tema) = catalog.themes.iter().find(|tema| tema.name == nome) else {
            panic!("{nome} ausente");
        };
        assert_eq!(tema.icons(), Some(icones));
        assert!(!tema.wallpapers().is_empty(), "{nome} sem papel de parede");
    }
    Ok(())
}
