use std::path::{Path, PathBuf};

use sparky_theme::locations::DEFAULT_THEME;

use crate::domain::system::themes::{Theme, Themes};
use crate::domain::system::{Control, Row, RowId};
use crate::domain::value::Step;

pub fn row(themes: &Themes, configured: Option<&str>, id: RowId) -> Option<Row> {
    let theme = current(themes)?;
    let total = theme.wallpapers().len();
    if total == 0 {
        return None;
    }
    let names = labels(theme);
    let value = match position(theme, configured) {
        Some(index) => {
            let mut args = fluent_bundle::FluentArgs::new();
            args.set("index", (index + 1) as i64);
            args.set("total", total as i64);
            args.set("name", names.get(index)?.as_str());
            crate::i18n::message("wallpaper-position", Some(&args))
        }
        None => crate::i18n::with("wallpaper-other", "total", &total.to_string()),
    };
    Some(Row {
        id,
        label: crate::i18n::text("group-wallpaper"),
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
