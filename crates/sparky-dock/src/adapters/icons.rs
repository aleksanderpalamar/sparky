use std::collections::HashMap;

use sparky_launcher::adapters::icons::Icons;
use sparky_launcher::app::pictures::{Shown, shown};
use sparky_launcher::domain::icon::Icon;

use crate::domain::app::App;

pub const ADD: &str = "list-add";
const ADD_NAMES: &[&str] = &["list-add-symbolic", "list-add"];

pub fn resolve(apps: &[App]) -> HashMap<String, Shown> {
    let mut wanted: Vec<(String, Icon)> = apps
        .iter()
        .filter_map(|app| Some((app.id.clone(), Icon::Named(app.icon.clone()?))))
        .collect();
    wanted.push((ADD.to_owned(), Icon::Symbolic(ADD_NAMES)));
    shown(Icons::current().named(&wanted))
}
