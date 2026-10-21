use std::fs;
use std::path::{Path, PathBuf};

use crate::locations;

const EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

pub fn is_image(path: &Path) -> bool {
    let extension = path.extension().and_then(|extension| extension.to_str()).map(str::to_ascii_lowercase);
    extension.is_some_and(|extension| EXTENSIONS.contains(&extension.as_str()))
}

pub fn images(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.retain(|path| is_image(path));
    paths.sort_by(|first, second| first.file_name().cmp(&second.file_name()));
    paths
}

pub fn of_theme(roots: &[PathBuf], theme: &str) -> Vec<PathBuf> {
    roots.iter().flat_map(|root| images(listing(&root.join(theme)))).collect()
}

pub fn roots(data_home: Option<&str>, home: &str, sparky_path: Option<&str>) -> Vec<PathBuf> {
    let data = match data_home.filter(|path| !path.trim().is_empty()) {
        Some(path) => PathBuf::from(path),
        None => Path::new(home).join(".local").join("share"),
    };
    vec![data.join("sparky").join("wallpapers"), locations::sparky_path(sparky_path).join("wallpapers")]
}

pub fn roots_from_env() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let data_home = std::env::var("XDG_DATA_HOME").ok();
    let sparky_path = std::env::var("SPARKY_PATH").ok();
    roots(data_home.as_deref(), &home, sparky_path.as_deref())
}

pub fn expand(path: &str, home: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => Path::new(home).join(rest),
        None => PathBuf::from(path),
    }
}

fn listing(folder: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(folder).into_iter().flatten().flatten();
    entries.map(|entry| entry.path()).filter(|path| path.is_file()).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{expand, images, is_image, of_theme, roots};

    #[test]
    fn so_imagens_contam_e_saem_em_ordem_de_nome() {
        assert!(is_image(&PathBuf::from("/a/1-lago.WEBP")));
        assert!(!is_image(&PathBuf::from("/a/intro.mp4")));
        let achadas = images(vec!["/t/2-b.jpg".into(), "/t/notas.txt".into(), "/t/1-a.png".into()]);
        assert_eq!(achadas, vec![PathBuf::from("/t/1-a.png"), PathBuf::from("/t/2-b.jpg")]);
    }

    #[test]
    fn os_do_usuario_vem_antes_dos_do_sistema() {
        let Ok(pasta) = tempfile::tempdir() else { panic!("sem pasta temporária") };
        let usuario = pasta.path().join("usuario");
        let sistema = pasta.path().join("sistema");
        for (raiz, nome) in [(&usuario, "9-meu.jpg"), (&sistema, "1-lago.webp"), (&sistema, "intro.mp4")] {
            assert!(fs::create_dir_all(raiz.join("nord")).is_ok());
            assert!(fs::write(raiz.join("nord").join(nome), b"x").is_ok());
        }
        let achadas = of_theme(&[usuario.clone(), sistema.clone(), pasta.path().join("ausente")], "nord");
        assert_eq!(achadas, vec![usuario.join("nord/9-meu.jpg"), sistema.join("nord/1-lago.webp")]);
    }

    #[test]
    fn as_pastas_seguem_o_xdg_e_o_sparky_path() {
        assert_eq!(roots(None, "/home/ana", None), vec![PathBuf::from("/home/ana/.local/share/sparky/wallpapers"), PathBuf::from("/usr/share/sparky/wallpapers")]);
        assert_eq!(roots(Some("/dados"), "/home/ana", Some("/opt/sparky"))[1], PathBuf::from("/opt/sparky/wallpapers"));
        assert_eq!(expand("~/fundo.jpg", "/home/ana"), PathBuf::from("/home/ana/fundo.jpg"));
        assert_eq!(expand("/fundo.jpg", "/home/ana"), PathBuf::from("/fundo.jpg"));
    }
}
