use std::path::{Path, PathBuf};

use sparky_xdg::{UserDir, UserDirs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub label: &'static str,
    pub path: PathBuf,
}

const FOLDERS: [(&str, UserDir); 6] = [
    ("Área de trabalho", UserDir::Desktop),
    ("Documentos", UserDir::Documents),
    ("Downloads", UserDir::Download),
    ("Imagens", UserDir::Pictures),
    ("Músicas", UserDir::Music),
    ("Vídeos", UserDir::Videos),
];

pub fn places(dirs: &UserDirs) -> Vec<Place> {
    let home = Place {
        label: "Pasta pessoal",
        path: dirs.home(),
    };
    let folders = FOLDERS.map(|(label, dir)| Place { label, path: dirs.get(dir) });
    std::iter::once(home)
        .chain(folders.into_iter().filter(|place| place.path != dirs.home()))
        .collect()
}

pub fn display(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{display, places};
    use sparky_xdg::UserDirs;
    use std::path::{Path, PathBuf};

    #[test]
    fn a_pasta_pessoal_vem_primeiro_e_as_outras_seguem_o_user_dirs() {
        let dirs = UserDirs::new("/home/p", Some("XDG_PICTURES_DIR=\"$HOME/Imagens\"\n".to_owned()));
        let lugares = places(&dirs);
        assert_eq!(lugares[0].label, "Pasta pessoal");
        assert_eq!(lugares[0].path, PathBuf::from("/home/p"));
        assert!(lugares.iter().any(|place| place.label == "Imagens" && place.path == Path::new("/home/p/Imagens")));
        assert_eq!(lugares.len(), 7);
    }

    #[test]
    fn pasta_xdg_desligada_aponta_para_o_home_e_nao_se_repete() {
        let dirs = UserDirs::new("/home/p", Some("XDG_DESKTOP_DIR=\"$HOME/\"\n".to_owned()));
        assert!(places(&dirs).iter().all(|place| place.label != "Área de trabalho"));
    }

    #[test]
    fn o_caminho_dentro_do_home_comeca_com_til() {
        let home = Path::new("/home/p");
        assert_eq!(display(Path::new("/home/p"), home), "~");
        assert_eq!(display(Path::new("/home/p/Fotos/2026"), home), "~/Fotos/2026");
        assert_eq!(display(Path::new("/etc"), home), "/etc");
    }
}
