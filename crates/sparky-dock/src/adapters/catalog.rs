use sparky_xdg::applications::installed_entries;
use sparky_xdg::desktop_entry::locale_from_env;

use crate::domain::app::App;

pub fn installed_apps() -> Vec<App> {
    let locale = locale_from_env();
    let mut apps: Vec<App> = installed_entries()
        .iter()
        .filter_map(|(id, entry)| App::of(id, entry, &locale))
        .collect();
    apps.sort_by(|left, right| sparky_ui::natural::compare(&left.name, &right.name));
    apps
}
