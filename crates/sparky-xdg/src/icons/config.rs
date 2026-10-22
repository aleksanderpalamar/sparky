use std::env;
use std::fs;
use std::path::PathBuf;

pub fn configured_theme() -> String {
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    let Some(config) = config else {
        return "hicolor".to_owned();
    };
    ["gtk-3.0", "gtk-4.0"]
        .iter()
        .filter_map(|gtk| fs::read_to_string(config.join(gtk).join("settings.ini")).ok())
        .find_map(|contents| theme_from_settings(&contents))
        .unwrap_or_else(|| "hicolor".to_owned())
}

pub fn icon_roots() -> Vec<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from);
    let mut roots: Vec<_> = home.as_ref().map(|home| home.join(".icons")).into_iter().collect();
    let data = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|home| home.join(".local/share")));
    roots.extend(data.map(|data| data.join("icons")));
    let shared = env::var_os("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    roots.extend(
        env::split_paths(&shared)
            .filter(|path| path.is_absolute())
            .map(|path| path.join("icons")),
    );
    roots
}

pub fn theme_from_settings(contents: &str) -> Option<String> {
    let mut current = false;
    for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
            current = line == "[Settings]";
            continue;
        }
        if !current {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "gtk-icon-theme-name" {
            continue;
        }
        let value = value.trim().trim_matches('"');
        return (!value.is_empty()).then(|| value.to_owned());
    }
    None
}
