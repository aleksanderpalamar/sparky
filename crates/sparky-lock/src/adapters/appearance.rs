use std::fs;
use std::path::{Path, PathBuf};

use sparky_login_core::appearance;

pub fn wallpaper(
    config: &Path,
    home: &Path,
    fallback: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    let text = fs::read_to_string(config).ok();
    appearance::wallpaper(text.as_deref(), home).or_else(fallback)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::wallpaper;

    fn padrao() -> Option<PathBuf> {
        Some(PathBuf::from(
            "/usr/share/sparky/wallpapers/tokyo-night/1.png",
        ))
    }

    #[test]
    fn usa_o_papel_de_parede_da_tela_de_login() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let config = dir.path().join("sparky.toml");
        assert!(
            fs::write(
                &config,
                "[general]\nwallpaper = \"/w/a.jpg\"\n\n[login]\nwallpaper = \"~/b.jpg\"\n"
            )
            .is_ok()
        );
        assert_eq!(
            wallpaper(&config, Path::new("/home/ana"), padrao),
            Some(PathBuf::from("/home/ana/b.jpg"))
        );
    }

    #[test]
    fn sem_arquivo_ou_sem_escolha_vale_o_padrao() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let config = dir.path().join("sparky.toml");
        assert_eq!(wallpaper(&config, Path::new("/home/ana"), padrao), padrao());
        assert!(fs::write(&config, "[login]\nwallpaper = \"\"\n").is_ok());
        assert_eq!(wallpaper(&config, Path::new("/home/ana"), padrao), padrao());
    }
}
