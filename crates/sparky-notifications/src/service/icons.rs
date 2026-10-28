use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

pub fn resolve(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    if let Some(path) = local(name) {
        return accepted(path).unwrap_or_default();
    }
    static CACHE: OnceLock<Mutex<Option<(String, IconTheme)>>> = OnceLock::new();
    let Ok(mut cache) = CACHE.get_or_init(|| Mutex::new(None)).lock() else {
        return String::new();
    };
    let theme = configured_theme();
    if cache.as_ref().is_none_or(|(current, _)| current != &theme) {
        *cache = Some((theme.clone(), IconTheme::new(icon_roots(), &theme)));
    }
    let Some((_, icons)) = cache.as_ref() else {
        return String::new();
    };
    let key = name.strip_suffix(".desktop").unwrap_or(name);
    icons.resolve(&[key]).and_then(accepted).unwrap_or_default()
}

fn local(value: &str) -> Option<PathBuf> {
    if Path::new(value).is_absolute() {
        return Some(PathBuf::from(value));
    }
    url::Url::parse(value).ok()?.to_file_path().ok()
}

fn accepted(path: PathBuf) -> Option<String> {
    let metadata = path.metadata().ok()?;
    let svg = path.extension().is_some_and(|extension| extension == "svg");
    let limit = if svg { 256 * 1024 } else { 4 * 1024 * 1024 };
    if !metadata.is_file() || metadata.len() > limit {
        return None;
    }
    Some(path.to_string_lossy().into_owned())
}
