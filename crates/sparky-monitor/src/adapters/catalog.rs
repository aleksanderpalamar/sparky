use std::path::Path;

use sparky_xdg::applications::installed_entries;
use sparky_xdg::desktop_entry::locale_from_env;
use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::apps::{AppEntry, program_of};
use crate::ports::Catalog;

const ICON_PIXELS: u32 = 24;

pub struct DesktopCatalog;

impl Catalog for DesktopCatalog {
    fn apps(&self) -> Vec<AppEntry> {
        let locale = locale_from_env();
        let theme = IconTheme::with_size(icon_roots(), &configured_theme(), ICON_PIXELS);
        installed_entries()
            .into_iter()
            .filter_map(|(id, entry)| {
                Some(AppEntry {
                    name: entry.name.get(&locale)?.to_owned(),
                    program: program_of(entry.exec.as_deref()?)?,
                    icon: entry.icon.and_then(|icon| icon_path(&theme, &icon)),
                    id,
                })
            })
            .collect()
    }
}

fn icon_path(theme: &IconTheme, icon: &str) -> Option<String> {
    if Path::new(icon).is_absolute() {
        return Path::new(icon).is_file().then(|| icon.to_owned());
    }
    theme
        .resolve(&[icon])
        .map(|path| path.to_string_lossy().into_owned())
}
