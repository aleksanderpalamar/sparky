use sparky_xdg::applications::installed_entries;
use sparky_xdg::desktop_entry::locale_from_env;

use crate::domain::app::{App, app_of};

const DESKTOP: &str = "sparky";

pub fn installed_apps() -> Vec<App> {
    let locale = locale_from_env();
    installed_entries()
        .into_iter()
        .filter_map(|(id, entry)| app_of(&id, &entry, &locale, DESKTOP))
        .collect()
}
