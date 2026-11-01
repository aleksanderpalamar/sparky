use std::path::{Path, PathBuf};

use sparky_theme::locations::DEFAULT_THEME;

use crate::domain::system::themes::{Theme, Themes};
use crate::domain::system::{Control, Row, RowId};
use crate::domain::value::Step;

const LABEL: &str = "Papel de parede";

pub fn row(themes: &Themes, configured: Option<&str>) -> Option<Row> {
    let theme = current(themes)?;
    let total = theme.wallpapers().len();
    if total == 0 {
        return None;
    }
    let names = labels(theme);
    let value = match position(theme, configured) {
        Some(index) => format!("{} de {total} · {}", index + 1, names.get(index)?),
        None => format!("{total} do tema · outro em uso"),
    };
    Some(Row {
        id: RowId::Wallpaper,
        label: LABEL.to_owned(),
        value,
        control: Control::Choice(names),
    })
}

pub fn chosen(themes: &Themes, label: &str) -> Option<PathBuf> {
    let theme = current(themes)?;
    let index = labels(theme)
        .iter()
        .position(|candidate| candidate == label.trim())?;
    theme.wallpapers().get(index).cloned()
}

pub fn step(themes: &Themes, configured: Option<&str>, step: Step) -> Option<PathBuf> {
    let theme = current(themes)?;
    let count = theme.wallpapers().len();
    if count == 0 {
        return None;
    }
    let next = match (position(theme, configured), step) {
        (Some(index), Step::More) => (index + 1) % count,
        (Some(index), Step::Less) => (index + count - 1) % count,
        (None, Step::More) => 0,
        (None, Step::Less) => count - 1,
    };
    theme.wallpapers().get(next).cloned()
}

fn current(themes: &Themes) -> Option<&Theme> {
    let Themes::Ready(catalog) = themes else {
        return None;
    };
    catalog
        .themes
        .iter()
        .find(|theme| theme.palette() == catalog.current)
}

fn position(theme: &Theme, configured: Option<&str>) -> Option<usize> {
    match configured {
        Some(path) => theme
            .wallpapers()
            .iter()
            .position(|wallpaper| wallpaper.as_path() == Path::new(path)),
        None if theme.name == DEFAULT_THEME => Some(0),
        None => None,
    }
}

fn labels(theme: &Theme) -> Vec<String> {
    let names: Vec<String> = theme
        .wallpapers()
        .iter()
        .map(|path| file_name(path))
        .collect();
    names
        .iter()
        .enumerate()
        .map(|(index, name)| match repeated(&names[..index], name) {
            0 => name.clone(),
            earlier => format!("{name} ({})", earlier + 1),
        })
        .collect()
}

fn repeated(earlier: &[String], name: &str) -> usize {
    earlier.iter().filter(|other| *other == name).count()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
