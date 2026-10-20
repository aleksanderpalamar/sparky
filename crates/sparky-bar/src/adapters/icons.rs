use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SIZES: [&str; 9] = [
    "22x22", "24x24", "scalable", "32x32", "16x16", "48x48", "64x64", "128x128", "256x256",
];
const CONTEXTS: [&str; 5] = ["apps", "status", "devices", "panel", "actions"];
const EXTENSIONS: [&str; 2] = ["png", "svg"];
const FALLBACK_THEMES: [&str; 2] = ["hicolor", "Adwaita"];

pub fn resolve(name: &str, theme_path: Option<&str>) -> Option<PathBuf> {
    let themes = themes(gtk_icon_theme().as_deref());
    candidates(name, theme_path, &data_dirs(), &themes)
        .into_iter()
        .find(|path| path.is_file())
}

pub fn candidates(name: &str, theme_path: Option<&str>, data_dirs: &[PathBuf], themes: &[String]) -> Vec<PathBuf> {
    if Path::new(name).is_absolute() {
        return vec![PathBuf::from(name)];
    }
    let mut found: Vec<PathBuf> = Vec::new();
    if let Some(directory) = theme_path.filter(|path| !path.is_empty()) {
        found.extend(with_extensions(&Path::new(directory).join(name)));
    }
    for directory in data_dirs {
        for theme in themes {
            for size in SIZES {
                for context in CONTEXTS {
                    let base = directory.join("icons").join(theme).join(size).join(context).join(name);
                    found.extend(with_extensions(&base));
                }
            }
        }
        found.extend(with_extensions(&directory.join("pixmaps").join(name)));
    }
    found
}

pub fn themes(configured: Option<&str>) -> Vec<String> {
    let mut themes: Vec<String> = configured.map(str::to_owned).into_iter().collect();
    themes.extend(FALLBACK_THEMES.iter().map(|theme| (*theme).to_owned()));
    themes.dedup();
    themes
}

pub fn icon_theme_from_settings(contents: &str) -> Option<String> {
    contents
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "gtk-icon-theme-name")
        .map(|(_, value)| value.trim().trim_matches('"').to_owned())
        .filter(|value| !value.is_empty())
}

fn with_extensions(base: &Path) -> Vec<PathBuf> {
    EXTENSIONS
        .iter()
        .map(|extension| base.with_extension(extension))
        .collect()
}

fn gtk_icon_theme() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    let contents = fs::read_to_string(Path::new(&config).join("gtk-3.0").join("settings.ini")).ok()?;
    icon_theme_from_settings(&contents)
}

fn data_dirs() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    let data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"));
    let shared = env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_owned());
    std::iter::once(data_home.as_str())
        .chain(shared.split(':'))
        .filter(|directory| !directory.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{candidates, icon_theme_from_settings, themes};
    use std::path::PathBuf;

    #[test]
    fn caminho_absoluto_e_o_proprio_icone() {
        assert_eq!(candidates("/opt/app/icone.png", None, &[], &[]), vec![PathBuf::from("/opt/app/icone.png")]);
    }

    #[test]
    fn o_diretorio_do_proprio_item_vem_primeiro() {
        let found = candidates("steam", Some("/opt/steam/icons"), &[PathBuf::from("/usr/share")], &themes(None));
        assert_eq!(found[0], PathBuf::from("/opt/steam/icons/steam.png"));
        assert_eq!(found[1], PathBuf::from("/opt/steam/icons/steam.svg"));
    }

    #[test]
    fn procura_no_tema_por_tamanho_e_contexto() {
        let found = candidates("discord", None, &[PathBuf::from("/usr/share")], &themes(None));
        assert_eq!(found[0], PathBuf::from("/usr/share/icons/hicolor/22x22/apps/discord.png"));
        assert!(found.contains(&PathBuf::from("/usr/share/icons/hicolor/scalable/status/discord.svg")));
        assert!(found.contains(&PathBuf::from("/usr/share/pixmaps/discord.png")));
    }

    #[test]
    fn o_tema_do_usuario_vem_antes_do_hicolor() {
        assert_eq!(themes(Some("Yaru-magenta")), vec!["Yaru-magenta", "hicolor", "Adwaita"]);
    }

    #[test]
    fn le_o_tema_de_icones_do_gtk() {
        let ini = "[Settings]\ngtk-theme-name=adw-gtk3-dark\ngtk-icon-theme-name=Yaru-magenta\n";
        assert_eq!(icon_theme_from_settings(ini).as_deref(), Some("Yaru-magenta"));
        assert_eq!(icon_theme_from_settings("[Settings]\n"), None);
    }
}
