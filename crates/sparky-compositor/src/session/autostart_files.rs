use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::session::autostart::Candidate;

const DEFAULT_CONFIG_DIRS: &str = "/etc/xdg";

pub fn directories(config_home: Option<&str>, home: &str, config_dirs: Option<&str>) -> Vec<PathBuf> {
    let filled = |value: Option<&str>| value.map(str::trim).filter(|text| !text.is_empty()).map(str::to_owned);
    let user = filled(config_home).unwrap_or_else(|| format!("{home}/.config"));
    let shared = filled(config_dirs).unwrap_or_else(|| DEFAULT_CONFIG_DIRS.to_owned());
    std::iter::once(user.as_str())
        .chain(shared.split(':'))
        .filter(|directory| !directory.is_empty())
        .map(|directory| Path::new(directory).join("autostart"))
        .collect()
}

pub fn from_system() -> Vec<Vec<Candidate>> {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    let config_dirs = env::var("XDG_CONFIG_DIRS").ok();
    directories(config_home.as_deref(), &home, config_dirs.as_deref())
        .iter()
        .map(|directory| read_directory(directory))
        .collect()
}

fn read_directory(directory: &Path) -> Vec<Candidate> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "desktop"))
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            let contents = fs::read_to_string(&path).ok()?;
            Some(Candidate { name, contents })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::directories;
    use std::path::PathBuf;

    #[test]
    fn o_autostart_do_usuario_vem_antes_do_sistema() {
        assert_eq!(
            directories(None, "/home/palamar", None),
            vec![
                PathBuf::from("/home/palamar/.config/autostart"),
                PathBuf::from("/etc/xdg/autostart"),
            ]
        );
    }

    #[test]
    fn segue_as_variaveis_xdg_quando_existem() {
        assert_eq!(
            directories(Some("/cfg"), "/home/palamar", Some("/etc/xdg:/usr/local/etc/xdg")),
            vec![
                PathBuf::from("/cfg/autostart"),
                PathBuf::from("/etc/xdg/autostart"),
                PathBuf::from("/usr/local/etc/xdg/autostart"),
            ]
        );
    }

    #[test]
    fn variavel_vazia_vale_o_padrao() {
        assert_eq!(directories(Some(" "), "/h", Some(""))[0], PathBuf::from("/h/.config/autostart"));
    }
}
