use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::{Application, Source, valid_id};

pub fn resolve(apps: &[Application], count: usize) -> HashMap<String, PathBuf> {
    let theme = IconTheme::with_size(icon_roots(), &configured_theme(), 128);
    apps.iter()
        .take(count.max(5))
        .filter_map(|app| find(&theme, app).map(|path| (app.id.clone(), path)))
        .collect()
}

fn find(theme: &IconTheme, app: &Application) -> Option<PathBuf> {
    if !valid_id(&app.id) {
        return None;
    }
    if matches!(app.source, Source::Flathub(_)) {
        for root in appstream_roots() {
            if let Some(path) = cached_icon(&root, &app.id, std::env::consts::ARCH) {
                return Some(path);
            }
        }
    }
    theme.resolve(&[&app.id])
}

fn appstream_roots() -> Vec<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    let mut roots = Vec::new();
    if let Some(data_home) = data_home {
        roots.push(data_home.join("flatpak"));
    }
    roots.push(PathBuf::from("/var/lib/flatpak"));
    roots
}

fn cached_icon(root: &Path, id: &str, arch: &str) -> Option<PathBuf> {
    let base = root
        .join("appstream/flathub")
        .join(arch)
        .join("active/icons");
    ["128x128", "64x64"]
        .into_iter()
        .map(|size| base.join(size).join(format!("{id}.png")))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::cached_icon;
    use std::fs;

    #[test]
    fn encontra_icone_appstream_sem_caminho_especifico_do_usuario() {
        let root = std::env::temp_dir().join(format!("sparky-store-icons-{}", std::process::id()));
        let icons = root.join("appstream/flathub/x86_64/active/icons/128x128");
        fs::create_dir_all(&icons).unwrap();
        let icon = icons.join("org.example.App.png");
        fs::write(&icon, b"image").unwrap();
        assert_eq!(
            cached_icon(&root, "org.example.App", "x86_64"),
            Some(icon.clone())
        );
        fs::remove_file(icon).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
