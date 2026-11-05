use std::path::PathBuf;

use sparky_xdg::{UserDir, UserDirs};

use crate::domain::icons::IconKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub key: &'static str,
    pub icon: IconKind,
    pub path: PathBuf,
}

const FOLDERS: [(&str, IconKind, UserDir); 6] = [
    ("place-desktop", IconKind::Desktop, UserDir::Desktop),
    ("place-documents", IconKind::Documents, UserDir::Documents),
    ("place-downloads", IconKind::Downloads, UserDir::Download),
    ("place-pictures", IconKind::Pictures, UserDir::Pictures),
    ("place-music", IconKind::Music, UserDir::Music),
    ("place-videos", IconKind::Videos, UserDir::Videos),
];

pub fn places(dirs: &UserDirs) -> Vec<Place> {
    let home = Place {
        key: "place-home",
        icon: IconKind::Home,
        path: dirs.home(),
    };
    let folders = FOLDERS.map(|(key, icon, dir)| Place {
        key,
        icon,
        path: dirs.get(dir),
    });
    std::iter::once(home)
        .chain(
            folders
                .into_iter()
                .filter(|place| place.path != dirs.home()),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::places;
    use crate::domain::icons::IconKind;
    use sparky_xdg::UserDirs;
    use std::path::{Path, PathBuf};

    #[test]
    fn a_pasta_pessoal_vem_primeiro_e_as_outras_seguem_o_user_dirs() {
        let dirs = UserDirs::new(
            "/home/p",
            Some("XDG_PICTURES_DIR=\"$HOME/Imagens\"\n".to_owned()),
        );
        let lugares = places(&dirs);
        assert_eq!(lugares[0].key, "place-home");
        assert_eq!(lugares[0].icon, IconKind::Home);
        assert_eq!(lugares[0].path, PathBuf::from("/home/p"));
        assert!(lugares.iter().any(|place| place.key == "place-pictures"
            && place.icon == IconKind::Pictures
            && place.path == Path::new("/home/p/Imagens")));
        assert_eq!(lugares.len(), 7);
    }

    #[test]
    fn pasta_xdg_desligada_aponta_para_o_home_e_nao_se_repete() {
        let dirs = UserDirs::new("/home/p", Some("XDG_DESKTOP_DIR=\"$HOME/\"\n".to_owned()));
        assert!(
            places(&dirs)
                .iter()
                .all(|place| place.key != "place-desktop")
        );
    }
}
