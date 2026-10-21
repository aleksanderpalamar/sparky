use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sparky_xdg::desktop_entry::{DesktopEntry, locale_from_env};
use sparky_xdg::programs::installed;

use crate::domain::app::{App, app_of};

const DESKTOP: &str = "sparky";
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

pub fn directories(data_home: Option<&str>, home: &str, data_dirs: Option<&str>) -> Vec<PathBuf> {
    let filled = |value: Option<&str>| value.map(str::trim).filter(|text| !text.is_empty()).map(str::to_owned);
    let user = filled(data_home).unwrap_or_else(|| format!("{home}/.local/share"));
    let shared = filled(data_dirs).unwrap_or_else(|| DEFAULT_DATA_DIRS.to_owned());
    std::iter::once(user.as_str())
        .chain(shared.split(':'))
        .filter(|directory| !directory.is_empty())
        .map(|directory| Path::new(directory).join("applications"))
        .collect()
}

pub fn desktop_id(relative: &Path) -> Option<String> {
    let text = relative.to_str()?;
    let stem = text.strip_suffix(".desktop")?;
    Some(stem.replace('/', "-"))
}

pub fn installed_apps() -> Vec<App> {
    let home = env::var("HOME").unwrap_or_default();
    let data_home = env::var("XDG_DATA_HOME").ok();
    let data_dirs = env::var("XDG_DATA_DIRS").ok();
    let locale = locale_from_env();
    let mut seen: BTreeMap<String, Option<App>> = BTreeMap::new();
    for directory in directories(data_home.as_deref(), &home, data_dirs.as_deref()) {
        for (id, path) in desktop_files(&directory) {
            if let Entry::Vacant(slot) = seen.entry(id) {
                let app = read(&path).and_then(|entry| app_of(slot.key(), &entry, &locale, DESKTOP));
                slot.insert(app);
            }
        }
    }
    seen.into_values().flatten().collect()
}

fn read(path: &Path) -> Option<DesktopEntry> {
    let entry = DesktopEntry::parse(&fs::read_to_string(path).ok()?)?;
    entry.installed(installed).then_some(entry)
}

fn desktop_files(root: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(items) = fs::read_dir(&directory) else {
            continue;
        };
        for path in items.filter_map(Result::ok).map(|item| item.path()) {
            match path.is_dir() {
                true => pending.push(path),
                false => {
                    let id = path.strip_prefix(root).ok().and_then(desktop_id);
                    found.extend(id.map(|id| (id, path)));
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{desktop_id, directories};
    use std::path::{Path, PathBuf};

    #[test]
    fn a_pasta_do_usuario_vem_antes_das_do_sistema() {
        assert_eq!(
            directories(None, "/home/p", None),
            vec![
                PathBuf::from("/home/p/.local/share/applications"),
                PathBuf::from("/usr/local/share/applications"),
                PathBuf::from("/usr/share/applications"),
            ]
        );
        assert_eq!(directories(Some("/d"), "/h", Some("/a:/b"))[1], PathBuf::from("/a/applications"));
    }

    #[test]
    fn o_id_troca_barra_por_hifen_e_ignora_o_que_nao_e_desktop() {
        assert_eq!(desktop_id(Path::new("kde/org.kde.dolphin.desktop")).as_deref(), Some("kde-org.kde.dolphin"));
        assert_eq!(desktop_id(Path::new("firefox.desktop")).as_deref(), Some("firefox"));
        assert_eq!(desktop_id(Path::new("mimeinfo.cache")), None);
    }
}
