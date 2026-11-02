use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::sessions::SessionFile;

const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";
const FOLDER: &str = "wayland-sessions";
const EXTENSION: &str = "desktop";

pub fn from_system() -> Vec<SessionFile> {
    let data_dirs = std::env::var("XDG_DATA_DIRS").ok();
    folders(data_dirs.as_deref())
        .iter()
        .flat_map(|folder| listing(folder))
        .collect()
}

pub fn folders(data_dirs: Option<&str>) -> Vec<PathBuf> {
    let dirs = data_dirs
        .filter(|dirs| !dirs.trim().is_empty())
        .unwrap_or(DEFAULT_DATA_DIRS);
    dirs.split(':')
        .filter(|dir| !dir.is_empty())
        .map(|dir| Path::new(dir).join(FOLDER))
        .collect()
}

fn listing(folder: &Path) -> Vec<SessionFile> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == EXTENSION)
        })
        .collect();
    paths.sort();
    paths.iter().filter_map(|path| read(path)).collect()
}

fn read(path: &Path) -> Option<SessionFile> {
    let id = path.file_stem()?.to_str()?.to_owned();
    let contents = fs::read_to_string(path).ok()?;
    Some(SessionFile { id, contents })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::folders;

    #[test]
    fn as_pastas_seguem_o_xdg_data_dirs_e_sem_ele_o_padrao() {
        assert_eq!(
            folders(Some("/opt/share:/usr/share")),
            vec![
                PathBuf::from("/opt/share/wayland-sessions"),
                PathBuf::from("/usr/share/wayland-sessions")
            ]
        );
        assert_eq!(
            folders(None),
            vec![
                PathBuf::from("/usr/local/share/wayland-sessions"),
                PathBuf::from("/usr/share/wayland-sessions")
            ]
        );
        assert_eq!(folders(Some(" ")), folders(None));
    }
}
